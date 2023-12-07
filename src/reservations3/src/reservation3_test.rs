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
use std::thread;
use std::time::{Duration};
use crate::CANCELLED;
use crate::CONFIRMED;
use crate::UNCONFIRMED;

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
    fn test_1user_1compound_0possible(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };
        let projectors = Facility { fac_type: PROJECTOR, capacity: 0, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        let projectors_arc = Arc::new(RwLock::new(projectors));
        
        // create user bookings
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }];
        start_users(vec![1], vec![true], vec![usr1_bookings], program_time.clone());

        thread::sleep(Duration::from_secs(2));

        let bookings = &rooms_arc.read().unwrap().bookings;
        let booking0_status = bookings[0].read().unwrap().status;


        // we expect this output because the projector is not available,
        // that means that the booking is cancelled or unconfirmed
        assert!(booking0_status == CANCELLED || booking0_status == UNCONFIRMED);

    }

    #[test]
    fn test_1user_1compound_1possible(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };
        let projectors = Facility { fac_type: PROJECTOR, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        let projectors_arc = Arc::new(RwLock::new(projectors));
        
        // create user bookings
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 25, end: 30, facility: projectors_arc.clone() }];
        start_users(vec![1], vec![true], vec![usr1_bookings], program_time.clone());

        thread::sleep(Duration::from_secs(2));

        // check how many rooms we have
        let len_rooms = &rooms_arc.read().unwrap().bookings.len();
        let len_projectors = &projectors_arc.read().unwrap().bookings.len();

        let mut confirmed_rooms = 0;
        let mut confirmed_projectors = 0;

        for i in 0..*len_rooms {
            if rooms_arc.read().unwrap().bookings[i].read().unwrap().status == CONFIRMED {
                confirmed_rooms += 1;
            }
        }
        for i in 0..*len_projectors {
            if projectors_arc.read().unwrap().bookings[i].read().unwrap().status == CONFIRMED {
                confirmed_projectors += 1;
            }
        }

        // we expect this output because the projector & room are available,
        // that means that the booking is confirmed
        assert_eq!(confirmed_rooms, 1);
        assert_eq!(confirmed_projectors, 1);
    }

    #[test]
    fn test_2users_2compounds_2possible_no_overlap(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));

        // create user bookings        
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];
        let usr2_bookings = vec![BookingSkeleton { start: 25, end: 30, facility: rooms_arc.clone() }];
        start_users(vec![1, 2], vec![false,true], vec![usr1_bookings, usr2_bookings], program_time.clone());

        thread::sleep(Duration::from_secs(2));

        // check how many rooms we have
        let len_rooms = &rooms_arc.read().unwrap().bookings.len();

        let mut confirmed_rooms = 0;

        for i in 0..*len_rooms {
            if rooms_arc.read().unwrap().bookings[i].read().unwrap().status == CONFIRMED {
                confirmed_rooms += 1;
            }
        }

        // we expect this output because the projectors & rooms are available,
        // that means that compound bookings are confirmed
        assert_eq!(confirmed_rooms, 2);
        assert_eq!(rooms_arc.read().unwrap().bookings.len(), 2);
        assert!(!overlap(&rooms_arc.read().unwrap().bookings[0].read().unwrap(), &rooms_arc.read().unwrap().bookings[1].read().unwrap()));

        let bookings = &rooms_arc.read().unwrap().bookings;
        let user_id_0 = bookings[0].read().unwrap().user.id;
        let user_id_1 = bookings[1].read().unwrap().user.id;
        assert!((user_id_0 == 1 && user_id_1 == 2) || (user_id_0 == 2 && user_id_1 == 1));
        
    }

    #[test]
    fn test_2users_2compounds_1vip_1possible(){
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 1, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        // create user bookings
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];
        let usr2_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }];
        start_users(vec![1, 2], vec![false,true], vec![usr1_bookings, usr2_bookings], program_time.clone());

        thread::sleep(Duration::from_secs(2));

        // check how many rooms we have
        let len = &rooms_arc.read().unwrap().bookings.len();
        let bookings = &rooms_arc.read().unwrap().bookings;
        let booking0_user_vip = bookings[0].read().unwrap().user.vip;


        // we expect this output because the projectors & rooms are available,
        // but only one user is vip, that means that compound bookings are confirmed only
        // for the vip user and the other one is cancelled or unconfirmed
        if *len == 1 {
            assert!(booking0_user_vip);
            assert!(bookings[0].read().unwrap().status == CONFIRMED);
        } else {
            let booking1_user_vip = bookings[1].read().unwrap().user.vip;
            let booking0_status = bookings[0].read().unwrap().status;
            assert!(booking1_user_vip);
            assert!(booking0_status == CANCELLED);
        }    

    }

    #[test]
    fn test_3users_8bookings_2compund_1vip_2possible(){
        
        // start program time
        let program_time = start_program_time();

        // create facilities
        let rooms = Facility { fac_type: ROOM, capacity: 2, bookings: Vec::new() };
        let projectors = Facility { fac_type: PROJECTOR, capacity: 2, bookings: Vec::new() };

        // generate arcs on RwLockes
        let rooms_arc = Arc::new(RwLock::new(rooms));
        let projectors_arc = Arc::new(RwLock::new(projectors));
        // create user bookings
        let usr1_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }];
        let usr2_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }];
        let usr3_bookings = vec![BookingSkeleton { start: 10, end: 20, facility: rooms_arc.clone() }, BookingSkeleton { start: 10, end: 20, facility: projectors_arc.clone() }];
        start_users(vec![1, 2, 3], vec![false, false, true], vec![usr1_bookings, usr2_bookings, usr3_bookings], program_time.clone());


        // write me here correct assertion based on previous tests
        thread::sleep(Duration::from_secs(6));

        // check how many rooms we have
        let len_rooms = &rooms_arc.read().unwrap().bookings.len();
        let len_projectors = &projectors_arc.read().unwrap().bookings.len();

        let mut confirmed_rooms = 0;
        let mut confirmed_projectors = 0;
        let mut cancelled_rooms = 0;
        let mut cancelled_projectors = 0;

        for i in 0..*len_rooms {
            if rooms_arc.read().unwrap().bookings[i].read().unwrap().status == CONFIRMED {
                confirmed_rooms += 1;
            }
            if rooms_arc.read().unwrap().bookings[i].read().unwrap().status == CANCELLED {
                cancelled_rooms += 1;
            }
        }

        for i in 0..*len_projectors {
            if projectors_arc.read().unwrap().bookings[i].read().unwrap().status == CONFIRMED {
                confirmed_projectors += 1;
            }
            if projectors_arc.read().unwrap().bookings[i].read().unwrap().status == CANCELLED {
                cancelled_projectors += 1;
            }
        }

        // we expect this output because the projectors & rooms are available,
        // but only one user is vip, that means that one compound of vip user is confirmed,
        // one of non-vip user is confirmed and the other one is cancelled or unconfirmed
        assert_eq!(confirmed_rooms, 2);
        assert_eq!(confirmed_projectors, 2);

        assert!((cancelled_rooms == 0 && cancelled_projectors == 0) || (cancelled_rooms == 1 && cancelled_projectors == 1));

    }

}