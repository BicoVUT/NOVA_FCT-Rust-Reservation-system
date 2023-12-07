#[cfg(test)]
use crate::ProgramTime;
use crate::start_program_time;
use crate::BookingSkeleton;
use crate::ROOM;
use crate::PROJECTOR;
use crate::Facility;
use crate::start_users;
use std::sync::{Arc, RwLock};
use crate::overlap;

mod tests {
    use super::*;

    #[test]
    fn test_get_current_time() {
        let program_time = ProgramTime { time: 0 };
        assert_eq!(program_time.get_current_time(), 0);
    }

    #[test]
    fn test_start_program_time() {
        let program_time = start_program_time();
        assert_eq!(program_time.read().unwrap().get_current_time(), 0);
    }

    #[test]
    fn test_1user_2bookings_1possible_overlap(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];
        start_users(vec![1], vec![usr1_bookings], program_time.clone());

        // we expect this output because the only one room is available,
        // and there is overlap between the two bookings
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 1);
    }

    #[test]
    fn test_1user_2bookings_2possible_no_overlap(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 25, end: 30, facility: rooms_arc.clone() }];
        start_users(vec![1], vec![usr1_bookings], program_time.clone());

        // we expect this output because the only one room is available,
        // but there is no overlap between the two bookings
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 2);
        assert!(!overlap(&rooms_arc.read().unwrap().bookings[0], &rooms_arc.read().unwrap().bookings[1]));

    }

    #[test]
    fn test_1user_2bookings_2possible_different_facilities(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };
        let projectors = Facility { fac_type: PROJECTOR, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        let projectors_arc = Arc::new(RwLock::new(projectors));
        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 25, end: 30, facility: projectors_arc.clone() }];
        start_users(vec![1], vec![usr1_bookings], program_time.clone());

        // we expect this output because the only one room is available,
        // and one projector is available, but there is no overlap between the two bookings
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 1);
        assert_eq!(projectors_arc.read().unwrap().bookings.len(), 1);
        assert!(!overlap(&rooms_arc.read().unwrap().bookings[0], &projectors_arc.read().unwrap().bookings[0]));

    }

    #[test]
    fn test_2users_2bookings_2possible_no_overlap(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];
        let usr2_bookings = vec![BookingSkeleton { start: 25, end: 30, facility: rooms_arc.clone() }];
        start_users(vec![1, 2], vec![usr1_bookings, usr2_bookings], program_time.clone());



        // we expect this output because the only one room is available,
        // but there is no overlap between the two bookings of 2 users
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 2);
        assert!(!overlap(&rooms_arc.read().unwrap().bookings[0], &rooms_arc.read().unwrap().bookings[1]));

        let bookings = &rooms_arc.read().unwrap().bookings;
        let user_id_0 = bookings[0].user.id;
        let user_id_1 = bookings[1].user.id;
        assert!((user_id_0 == 1 && user_id_1 == 2) || (user_id_0 == 2 && user_id_1 == 1));
        
    }

    #[test]
    fn test_2users_2bookings_2possible_different_facilities(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };
        let projectors = Facility { fac_type: PROJECTOR, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        let projectors_arc = Arc::new(RwLock::new(projectors));
        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];
        let usr2_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }];
        start_users(vec![1, 2], vec![usr1_bookings, usr2_bookings], program_time.clone());

        // we expect this output because the only one room is available,
        // and one projector is available, but there is no overlap 
        //between the two bookings of 2 users
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 1);
        assert_eq!(projectors_arc.read().unwrap().bookings.len(), 1);
        assert!(overlap(&rooms_arc.read().unwrap().bookings[0], &projectors_arc.read().unwrap().bookings[0]));   
    }

    #[test]
    fn test_2users_2bookings_1possible_overlap(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];
        let usr2_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];

        start_users(vec![1, 2], vec![usr1_bookings, usr2_bookings], program_time.clone());

        // we expect this output because the only one room is available,
        // but there is overlap between the two bookings of 2 users
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 1);
        let bookings = &rooms_arc.read().unwrap().bookings;
        let user_id_0 = bookings[0].user.id;
        assert!(user_id_0 == 1 || user_id_0 == 2);

    }

    #[test]
    fn test_3users_8bookings_6possible(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 2, bookings: Vec::new() };
        let projectors = Facility { fac_type: PROJECTOR, capacity: 2, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        let projectors_arc = Arc::new(RwLock::new(projectors));
        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone()}, BookingSkeleton { start: 25, end: 30, facility: rooms_arc.clone() }];
        let usr2_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }, BookingSkeleton { start: 25, end: 30, facility: projectors_arc.clone()} ];
        let usr3_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }];
        start_users(vec![1, 2, 3], vec![usr1_bookings, usr2_bookings, usr3_bookings], program_time.clone());

        // we expect this output because 2 rooms and 2 projectors are available,
        // and there is overlap on some bookings so in total 6 bookings are possible
        // 3 bookings for rooms and 3 bookings for projectors
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 3); 
        assert_eq!(projectors_arc.read().unwrap().bookings.len(), 3);
    }

}