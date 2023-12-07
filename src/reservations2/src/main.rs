///////////////////////////////////////////////////////////////////////
//////////////// Simple Reservations System (Task 2) //////////////////
///////////////////////////////////////////////////////////////////////

// System:  The base system is the same as in Task 1, but now we have
//          vip- and non-vip users where bookings of vip users
//          lead to the cancellation of non-vip bookings if necessary.
//          VIPs cannot overwrite other VIPs' bookings.

// Implementation:  Different from before users now have an inbox on
//                  which they receive cancellation messages (a channel).
//                  Bookings now have a status (unconfirmed, confirmed, cancelled),
//                  where on cancellation the status of the booking (in the list of bookings
//                  of the facility) is changed to cancelled and the user notified.
//                  The facility keeps all bookings but only confirmed bookings are counted
//                  in the capacity checks.

///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod reservation2_test;

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

// A booking has a start and end time, a facility, a user and a status.
// The status can be unconfirmed, confirmed or cancelled and is changed
// as necessary.
struct Booking {
    start: u32,
    end: u32,
    facility: Arc<RwLock<Facility>>,
    user: Arc<User>,
    status: BookingStatus
}

// Booking skeleton
struct BookingSkeleton {
    start: u32,
    end: u32,
    facility: Arc<RwLock<Facility>>,
}

// A user has an id, a vip status and an inbox (channel) for cancellation messages.
// On which others can send. The channel for receiving is handed to the user function
// as an argument.
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

// This function starts the users with each living in a separate thread. Each user is given a list of bookings
// to try to book.
fn start_users(user_ids: Vec<u32>, user_stati: Vec<bool>, bookings: Vec<Vec<BookingSkeleton>>, program_time: Arc<RwLock<ProgramTime>>) {
    // start the user threads
    let threads: Vec<_> = (1..=user_ids.len()).enumerate().map(|(i, user_id)| {

        // create the channel for receiving / sending cancellation messages
        let (tx, rx) = mpsc::channel();
        let user = Arc::new(User { id: user_id as u32, vip: user_stati[i], adress: tx });

        // create list of bookings of the user from the booking skeletons
        let mut user_bookings: Vec<Arc<RwLock<Booking>>> = Vec::new();
        for booking in &bookings[i] {
            let user = Arc::clone(&user);
            let booking = Booking { start: booking.start, end: booking.end, user: user, facility: booking.facility.clone(), status: UNCONFIRMED };
            user_bookings.push(Arc::new(RwLock::new(booking)));
        }

        // get the user a reference to the program time
        let program_time = Arc::clone(&program_time);

        // reference to the bookings
        let user_bookings = Arc::new(user_bookings);

        // start the user thread
        thread::spawn(move || {
            run_user(user_bookings, program_time, rx);
        })
    }).collect();
    // drop(bookings);
    // joining the threads is a bit more difficult as all possible senders have to go out of scope
    // to let the drain from the notification channel end, which would require further effort
    // we did non feel necessary as the system "in the wild" would just run forever.
    return;
}

// This function runs a user. It tries to book the facilities in the list of bookings.
// Cancellation messages are received on the inbox.
fn run_user(to_book: Arc<Vec<Arc<RwLock<Booking>>>>, program_time: Arc<RwLock<ProgramTime>>, inbox: mpsc::Receiver<Arc<RwLock<Booking>>>) {
    for b in to_book.iter() {
        book_facility(b.clone(), program_time.clone());
        // now the user might react to the success of the booking
    }
    // drop(to_book);
    // wait for cancel messages
    for msg in inbox {
        let msg = msg.read().unwrap();
        // print user X received cancel message
        println!("❌: {} User {} received cancellation message.", vip_bool_to_string(msg.user.vip), msg.user.id);
    }
    // we should reach this poin if all possible senders go out of scope
}

/////////////////////// Booking function /////////////////////

// This function books a facility for a user at a given time, if available.
// It locks the facility and alters the bookings list of the facility,
// if possible. It returns true if the booking was successful and false otherwise.
// It receives the respective RwLocks as arguments.
fn book_facility(booking: Arc<RwLock<Booking>>, program_time: Arc<RwLock<ProgramTime>>) -> bool {
    {
        // lock the booking
        let booking_read = booking.write().unwrap();

        // lock the facility
        let mut facility = booking_read.facility.write().unwrap();

        // check if the booking is in the future
        if booking_read.start < program_time.read().unwrap().get_current_time() {
            // print User X couldn't book facility Y from time Z to time W - time in the past (current time is T)
            println!("❌: {} User {} couldn't book {} from time {} to time {} - time in the past (current time is {}).", vip_bool_to_string(booking_read.user.vip), booking_read.user.id, facility_type_to_string(facility.fac_type), booking_read.start, booking_read.end, program_time.read().unwrap().get_current_time());
            return false;
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
        // one of them is cancelled
        if booking_read.user.vip && overlaps >= facility.capacity && premium_overlaps < facility.capacity {
            // cancel the booking of a non-vip user
            for b in &facility.bookings {
                let mut bmut = b.write().unwrap();
                if overlap(&bmut, &booking_read) && !bmut.user.vip && bmut.status == CONFIRMED {
                    println!("❌: User {}'s booking of facility {} from time {} to time {} was cancelled as of a vip booking.", bmut.user.id, facility_type_to_string(facility.fac_type), bmut.start, bmut.end);
                    bmut.status = CANCELLED;
                    bmut.user.adress.send(b.clone()).unwrap();
                    break;
                }
            }
        } 
        
        // if the user is non-vip and the capacity is exceeded, decline the booking
        // if the user is vip but all bookings are vip and the capacity is exceeded, decline the booking
        if (overlaps >= facility.capacity && !booking_read.user.vip) || (booking_read.user.vip && premium_overlaps >= facility.capacity) {
            println!("❌: {} User {} couldn't book {} from time {} to time {} - capacity exceeded.", vip_bool_to_string(booking_read.user.vip), booking_read.user.id, facility_type_to_string(facility.fac_type), booking_read.start, booking_read.end);
            return false;
        }

        // here the booking can be done
        facility.bookings.push(booking.clone());

        // print success message
        println!("✅: {} User {} booked {} from time {} to time {}.", vip_bool_to_string(booking_read.user.vip), booking_read.user.id, facility_type_to_string(facility.fac_type), booking_read.start, booking_read.end);
    }

    // change the status of the booking to confirmed
    let mut booking_mut = booking.write().unwrap();
    booking_mut.status = CONFIRMED;
    
    return true;
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
    let usr1_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }];
    let usr2_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }];
    let usr3_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }];
    let usr4_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }];
    let usr5_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }];
    
    // start the users
    start_users(vec![1, 2, 3, 4, 5], vec![false, false, true, true, true], vec![usr1_bookings, usr2_bookings, usr3_bookings, usr4_bookings, usr5_bookings], program_time.clone());

    // wait for 10 seconds, joining the threads as previously is more
    // complicated as of the cancellation messages being received on the inboxes
    thread::sleep(Duration::from_secs(10));

    println!("=========== Program ended ===========");
}