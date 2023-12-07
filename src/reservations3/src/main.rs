///////////////////////////////////////////////////////////////////////
//////////////// Simple Reservations System (Task 3) //////////////////
///////////////////////////////////////////////////////////////////////

// System:  Bookings can compared to the previous version be made in
//          compounds consisting of different resources (e.g. a room and a projector).
//          A compound is only booked if all parts of it are possible.
//          If a part of a compound has to be cancelled due to a VIP request
//          the whole compound is cancelled and the user notified on all
//          necessary cancellations.

// Implementation:  Compounds can only be booked sequentially (see comment in the code).
//                  For all parts of the compound it is checked whether they are possible
//                  and if necessary what bookings have to be cancelled.
//                  If a cancellation is necessary, the booking is added to
//                  a list of bookings to be cancelled.

//                  Each booking now also references its compound, allowing for
//                  the cancellation of all bookings in the compound if one of
//                  them has to be cancelled.

//                  The respective actions after the check are done in the user
//                  thread; note that a server-client architecture as in a message
//                  passing system where only the server makes changes is not necessary
//                  here.


///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod reservation3_test;

use iota::iota;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;

//////////////////// Definition of useful Constants ////////////////////

type FacilityType = u32;
type BookingStatus = u32;

iota! {
    const ROOM: FacilityType = 1 << iota;
        , PROJECTOR
}

iota! {
    const UNCONFIRMED: BookingStatus = 1 << iota;
        , CONFIRMED
        , CANCELLED
}

//////////////////// Definition of useful Structs ////////////////////

// A facility has a type, a capacity and a list of bookings.
struct Facility {
    fac_type: FacilityType,
    capacity: u32,
    bookings: Vec<Arc<RwLock<Booking>>>,
}

// A booking has a start and end time, a facility, a user, a status
// and also references the compound it is part of.
struct Booking {
    start: u32,
    end: u32,
    facility: Arc<RwLock<Facility>>,
    user: Arc<User>,
    status: BookingStatus,
    compound: Option<Arc<Vec<Arc<RwLock<Booking>>>>>,
}

// Booking skeleton
struct BookingSkeleton {
    start: u32,
    end: u32,
    facility: Arc<RwLock<Facility>>,
}

// A user has an id, a vip status and an inbox.
struct User {
    id: u32,
    vip: bool,
    adress: mpsc::Sender<Arc<RwLock<Booking>>>
}

// ProgramTime
struct ProgramTime {
    time: u32,
}

////////////////// Timer function ///////////////////

impl ProgramTime {
    fn get_current_time(&self) -> u32 {
        self.time
    }
}

// Our program time is started and the Arc to the RwLock of the ProgramTime is returned
fn start_program_time() -> Arc<RwLock<ProgramTime>> {
    // Create a shared state for ProgramTime using Arc and RwLock
    let program_time = Arc::new(RwLock::new(ProgramTime { time: 0 }));

    // Clone Arc for the closure
    let program_time_clone = program_time.clone();

    // Create a thread to increment program time
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let now = Instant::now();
            let elapsed = now.duration_since(last_tick);
            if elapsed >= Duration::from_millis(100) {
                last_tick = now;
                let mut program_time = program_time_clone.write().unwrap();
                program_time.time += 1;
            }
        }
    });

    program_time
}


/////////////////////// Helpers /////////////////////

// This functions checks if two bookings overlap.
// It returns true if they overlap and false otherwise.
fn overlap(b1: &Booking, b2: &Booking) -> bool {
    if b1.start < b2.start {
        return b1.end > b2.start;
    } else {
        return b2.end > b1.start;
    }
}

// This function converts a facility type to a string.
fn facility_type_to_string(fac_type: FacilityType) -> String {
    match fac_type {
        ROOM => "Room".to_string(),
        PROJECTOR => "Projector".to_string(),
        _ => "Unknown".to_string(),
    }
}

// This function converts a vip bool to a string.
fn vip_bool_to_string(vip: bool) -> String {
    match vip {
        true => "VIP".to_string(),
        false => "Non-VIP".to_string(),
    }
}

/////////////////////// User server /////////////////////


// Notice that as of cancellation and the vip system a booking of any ressource (e.g. projector) can sideeffect any other ressource
// as in the compound of the cancelled e.g. projector everything else can be. Now, compounds have to be booked as an atomic unit
// because otherwise conflicts can arise, so bookings of compounds have to be done sequentially.
// Only the parts of the compound could be checked in parallel.

fn start_users(user_ids: Vec<u32>, user_stati: Vec<bool>, bookings: Vec<Vec<BookingSkeleton>>, program_time: Arc<RwLock<ProgramTime>>) {
    // Following the note above, we make sure that only once compound is booked at a time
    // using this Arc to a RwLock signaling if a compound is currently in process.
    let compound_in_process = Arc::new(RwLock::new(false));

    let threads: Vec<_> = (1..=user_ids.len()).enumerate().map(|(i, user_id)| {

        // create a channel for the reception and transmission of cancellation messages
        let (tx, rx) = mpsc::channel();
        let user = Arc::new(User { id: user_id as u32, vip: user_stati[i], adress: tx });

        // create list of bookings of the user from the booking skeletons
        let mut user_bookings: Vec<Arc<RwLock<Booking>>> = Vec::new();
        for booking in &bookings[i] {
            let user = Arc::clone(&user);
            let booking = Booking { start: booking.start, end: booking.end, user: user, facility: booking.facility.clone(), status: UNCONFIRMED, compound: None};
            user_bookings.push(Arc::new(RwLock::new(booking)));
        }

        // get the user a reference to the program time
        let program_time = Arc::clone(&program_time);

        // reference to the bookings
        let user_bookings = Arc::new(user_bookings);
        let compound_in_process = Arc::clone(&compound_in_process);

        // make each booking aware of the compound it is part of
        for booking in user_bookings.iter() {
            let mut booking_mut = booking.write().unwrap();
            booking_mut.compound = Some(user_bookings.clone());
        }

        // start the user
        thread::spawn(move || {
            run_user(user_bookings, program_time, rx, compound_in_process);
        })
    }).collect();
    return;
}

fn run_user(to_book: Arc<Vec<Arc<RwLock<Booking>>>>, program_time: Arc<RwLock<ProgramTime>>, inbox: mpsc::Receiver<Arc<RwLock<Booking>>>, compound_in_process: Arc<RwLock<bool>>) {
    // here we do one compound booking per user
    {
        // this reflects if the compound booking is possible
        let mut possible = true;

        // list of all bookings to be cancelled
        let mut cancel_list: Vec<Arc<RwLock<Booking>>> = Vec::new();

        // currently each user books only one compound, so we lock the compound_in_process here
        let mut compound_in_process = compound_in_process.write().unwrap();
        // and set it to true
        *compound_in_process = true;

        // As threads in Rust are heavier than say goroutines we provide a sequential and
        // a concurrent version of the booking check, where the overhead of the concurrent
        // version might make it the inferior choice in this case.

        // /////////////////////// Sequential Booking Check  /////////////////////
        // // we go over all bookings of the compound and check if they are bookable
        // // and what cancellations would have to be made
        // for b in to_book.iter() {

        //     // this could be done in a thread to enable concurrency
        //     let (success, to_cancel) = check_facility(b.clone(), program_time.clone());
            
        //     // if a cancellation is necessary, add it to the cancel list
        //     if let Some(b) = to_cancel {
        //         cancel_list.push(b);
        //     }

        //     // update the possible bool
        //     possible = possible && success;
        // }
        // /////////////////////////////////////////////////////////////////////
        
        ///////////////// Concurrent booking check //////////////////
        let mut handles = Vec::new();

        // we go over all bookings of the compound and check if they are bookable
        // and what cancellations would have to be made
        for b in to_book.iter() {
            let b = Arc::clone(b);
            let pt = Arc::clone(&program_time);
            let handle = thread::spawn(move || {
                let (success, to_cancel) = check_facility(b, pt);
                (success, to_cancel)
            });
            
            handles.push(handle)
        }

        // wait for all checks to be done
        for handle in handles {
            let (success, to_cancel) = handle.join().unwrap();
            // if a cancellation is necessary, add it to the cancel list
            if let Some(b) = to_cancel {
                cancel_list.push(b);
            }

            // update the possible bool
            possible = possible && success;
        }
        ///////////////////////////////////////////////////////////

        // if the compound is possible, book all of its parts
        // and cancel all bookings in the cancel list as well
        // all bookings in the compound of the conflicting bookings
        if possible {
            for b in to_book.iter() {
                {    
                    let mut bmut = b.write().unwrap();
                    bmut.status = CONFIRMED;
                }
            }
            // cancel all bookings in the cancel list
            for b in cancel_list {
                
                let mut bmut = b.write().unwrap();

                // cancel the conflicting booking
                if  bmut.status != CANCELLED {
                    bmut.status = CANCELLED;
                    println!("❌: {} User {}'s booking of facility {} from time {} to time {} was cancelled as of a vip booking.", vip_bool_to_string(bmut.user.vip), bmut.user.id, facility_type_to_string(bmut.facility.read().unwrap().fac_type), bmut.start, bmut.end);
                    bmut.user.adress.send(b.clone()).unwrap();
                }

                // cancel all bookings in the compound of the conflicting booking
                if let Some(compound) = &bmut.compound {
                    for b in compound.iter() {
                        if b.try_write().is_ok() { // this is to exclude the booking itself that is also part of the compound
                                                   // alternatively the construction of the compound could be changed
                            let mut bmut = b.write().unwrap();
                            if bmut.status != CANCELLED {
                                bmut.status = CANCELLED;
                                println!("❌: {} User {}'s booking of facility {} from time {} to time {} was cancelled as of a vip booking.", vip_bool_to_string(bmut.user.vip), bmut.user.id, facility_type_to_string(bmut.facility.read().unwrap().fac_type), bmut.start, bmut.end);
                                bmut.user.adress.send(b.clone()).unwrap();
                            }
                        }
                    }
                }
  
            }
            // print the success messages of all bookings in the compound
            for b in to_book.iter() {
                let b = b.read().unwrap();
                println!("✅: {} User {} booked facility {} from time {} to time {}.", vip_bool_to_string(b.user.vip), b.user.id, facility_type_to_string(b.facility.read().unwrap().fac_type), b.start, b.end);
            }
            // print a compound message
            println!("✅: {} User {} successfully booked all facilities.", vip_bool_to_string(to_book[0].read().unwrap().user.vip), to_book[0].read().unwrap().user.id);
        }
        else{
            // print failure message
            println!("❌: {} User {} couldn't book all facilities.", vip_bool_to_string(to_book[0].read().unwrap().user.vip), to_book[0].read().unwrap().user.id);
        }
        // set compound_in_process to false
        *compound_in_process = false;
    } // here the compound_in_process lock is released and the next user can book a compound

    // wait for cancellation messages
    for msg in inbox {
        let msg = msg.read().unwrap();
        // print user X received cancel message
        println!("❌: {} User {} received cancellation message.", vip_bool_to_string(msg.user.vip), msg.user.id);
    }
}

/////////////////////// Booking checker /////////////////////

// This function checks if a booking is possible and if necessary what conflicting booking has to be cancelled.
fn check_facility(booking: Arc<RwLock<Booking>>, program_time: Arc<RwLock<ProgramTime>>) -> (bool, Option<Arc<RwLock<Booking>>>) {
    
    let mut to_cancel: Option<Arc<RwLock<Booking>>> = None;

    // lock the booking
    let booking_read = booking.write().unwrap();

    // lock the facility
    let mut facility = booking_read.facility.write().unwrap();

    // check if the booking is in the future
    if booking_read.start < program_time.read().unwrap().get_current_time() {
        return (false, to_cancel);
    }

    // count the overlaps and the premium overlaps
    let mut overlaps = 0;
    let mut premium_overlaps = 0;  
    for b in &facility.bookings {
        let b = b.read().unwrap();
        if overlap(&b, &booking_read) && b.status == CONFIRMED {
            overlaps += 1;
            if b.user.vip {
                premium_overlaps += 1;
            }
        }
    }

    // if the user is a vip, we are at the capacity limit but there are non-vip bookings
    // one of them is a candidate for cancellation should the compund the booking is in be possible
    if booking_read.user.vip && overlaps >= facility.capacity && premium_overlaps < facility.capacity {
        for b in &facility.bookings {
            let mut bmut = b.write().unwrap();
            if overlap(&bmut, &booking_read) && !bmut.user.vip && bmut.status == CONFIRMED {
                to_cancel = Some(b.clone());
                break;
            }
        }
    } 
    
    // if the user is non-vip and the capacity is exceeded, decline the booking
    // if the user is vip but all bookings are vip and the capacity is exceeded, decline the booking
    if (overlaps >= facility.capacity && !booking_read.user.vip) || (booking_read.user.vip && premium_overlaps >= facility.capacity) {
        return (false, to_cancel);
    }

    // here the booking can be pushed to the facility
    // note that the status is only changed to confirmed
    // when the whole compound is possible
    facility.bookings.push(booking.clone());

    return (true, to_cancel);
}


/////////////////////// Main | initial tests /////////////////////

fn main() {

    // start program time
    let program_time = start_program_time();
    println!("=========== Program started ===========");

    // create facilities
    let rooms = Facility { fac_type: ROOM, capacity: 2, bookings: Vec::new() };
    let projectors = Facility { fac_type: PROJECTOR, capacity: 2, bookings: Vec::new() };
    let rooms_arc = Arc::new(RwLock::new(rooms));
    let projectors_arc = Arc::new(RwLock::new(projectors));

    // create example bookings
    let usr1_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }];
    let usr2_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }];
    let usr3_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }];
    start_users(vec![1, 2, 3], vec![false, false, true], vec![usr1_bookings, usr2_bookings, usr3_bookings], program_time.clone());

    // wait for 10 seconds; in the real world this system would just run forever
    thread::sleep(Duration::from_secs(10));

    println!("=========== Program ended ===========");
}