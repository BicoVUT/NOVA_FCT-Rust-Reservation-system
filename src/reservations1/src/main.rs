///////////////////////////////////////////////////////////////////////
//////////////// Simple Reservations System (Task 1) //////////////////
///////////////////////////////////////////////////////////////////////

// System:  This program implements a simple reservations system.
//          There are facilities (e.g. rooms or projectors) which
//          can be booked by users. The facilities have a capacity
//          and can only be booked if the capacity is not exceeded.

// Implementation: Each facility is managed as a struct with a list
//                 of bookings. When a user tries to book a certain facility
//                 e.g. a room, the facility is locked and the list of bookings
//                 is checked for overlaps. If the capacity is not exceeded,
//                 the booking is added to the list of bookings and the success
//                 is returned to the user.

//                 Each booking has a start and end time and also references
//                 the user and the facility, so the correct facility can easily
//                 be accessed and user information be used.

//                 Users run in different threads and try to book facilities, ressource
//                 management is done based on Arcs, RwLocks and Rusts ownership system.

///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod reservation1_test;

use iota::iota;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

//////////////////// Definition of useful Constants ////////////////////

type FacilityType = u32;

iota! {
    const ROOM: FacilityType = 1 << iota;
        , PROJECTOR
}

//////////////////// Definition of useful Structs ////////////////////

// A facility has a type, a capacity and a list of bookings.
struct Facility {
    fac_type: FacilityType,
    capacity: u32,
    bookings: Vec<Arc<Booking>>,
}

// A booking has a start and end time and references the user and the facility.
struct Booking {
    start: u32,
    end: u32,
    facility: Arc<RwLock<Facility>>,
    user: Arc<User>,
}

// The booking skeleton is used to create bookings that are handed to the user
// when we start the user and which of this compared to the booking to not
// have a user reference yet.
struct BookingSkeleton {
    start: u32,
    end: u32,
    facility: Arc<RwLock<Facility>>,
}

// A user has an id.
struct User {
    id: u32,
}

// ProgramTime struct, we use and Arc and RwLock to share it between threads
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

/////////////////////// User server /////////////////////

// This function starts the users with each living in a separate thread. Each user is given a list of bookings
// to try to book.
fn start_users(user_ids: Vec<u32>, bookings: Vec<Vec<BookingSkeleton>>, program_time: Arc<RwLock<ProgramTime>>) {

    // start the user threads
    let threads: Vec<_> = (1..=user_ids.len()).enumerate().map(|(i, user_id)| {
        // create the user
        let user = Arc::new(User { id: user_id as u32 });

        // create list of bookings of the user from the booking skeletons
        let mut user_bookings: Vec<Arc<Booking>> = Vec::new();
        for booking in &bookings[i] {
            let user = Arc::clone(&user);
            let booking = Booking { start: booking.start, end: booking.end, user: user, facility: booking.facility.clone() };
            user_bookings.push(Arc::new(booking));
        }

        // get the user a reference to the program time
        let program_time = Arc::clone(&program_time);

        // start the user thread
        thread::spawn(move || {
            run_user(Arc::new(user_bookings), program_time);
        })
    }).collect();
    for thread in threads {
        // wait for all users to finish the respective task
        thread.join().unwrap();
    }
    return;
}

// This function runs a user. It tries to book the facilities in the list of bookings.
fn run_user(to_book: Arc<Vec<Arc<Booking>>>, program_time: Arc<RwLock<ProgramTime>>) {
    for b in to_book.iter() {
        book_facility(b.clone(), program_time.clone());
        // now the user might react to the success of the booking
    }
}

/////////////////////// Booking function /////////////////////

// This function books a facility for a user at a given time, if available.
// It locks the facility and alters the bookings list of the facility,
// if possible. It returns true if the booking was successful and false otherwise.
// It receives the respective RwLocks as arguments.
fn book_facility(booking: Arc<Booking>, program_time: Arc<RwLock<ProgramTime>>) -> bool {

    // lock the facility
    let mut facility = booking.facility.write().unwrap();

    // check if the booking is in the future
    if booking.start < program_time.read().unwrap().get_current_time() {
        println!("❌: User {} couldn't book {} from time {} to time {} - time in the past (current time is {}).", booking.user.id, facility_type_to_string(facility.fac_type), booking.start, booking.end, program_time.read().unwrap().get_current_time());
        return false;
    }

    // check for possible overlaps of the booking
    let mut overlaps = 0;   
    for b in &facility.bookings {
        if overlap(b, &booking) {
            overlaps += 1;
        }
    }
    // if the capacity is exceeded, decline the booking
    if overlaps >= facility.capacity {
        // print User X couldn't book facility Y from time Z to time W - capacity exceeded.
        println!("❌: User {} couldn't book {} from time {} to time {} - capacity exceeded.", booking.user.id, facility_type_to_string(facility.fac_type), booking.start, booking.end);
        return false;
    }

    // here the booking can be done
    facility.bookings.push(booking.clone());

    // print success message
    println!("✅: User {} booked {} from time {} to time {}.", booking.user.id, facility_type_to_string(facility.fac_type), booking.start, booking.end);
    return true;
}


/////////////////////// Main | initial tests /////////////////////

fn main() {

    // start program time
    let program_time = start_program_time();
    println!("=========== Program started ===========");

    // create the facilities with respective references on RwLocks
    let rooms = Facility { fac_type: ROOM, capacity: 2, bookings: Vec::new() };
    let projectors = Facility { fac_type: PROJECTOR, capacity: 2, bookings: Vec::new() };
    let rooms_arc = Arc::new(RwLock::new(rooms));
    let projectors_arc = Arc::new(RwLock::new(projectors));
    
    // some example bookings
    let usr1_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 2, end: 4, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }];
    let usr2_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: projectors_arc.clone() }, BookingSkeleton { start: 1, end: 3, facility: projectors_arc.clone() }];
    let usr3_bookings = vec![BookingSkeleton { start: 1, end: 2, facility: rooms_arc.clone() }, BookingSkeleton { start: 1, end: 5, facility: projectors_arc.clone() }];
    
    // start the users
    start_users(vec![1, 2, 3], vec![usr1_bookings, usr2_bookings, usr3_bookings], program_time.clone());

    println!("=========== Program ended ===========");
}