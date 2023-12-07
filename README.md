# A Reservation System in Rust

In the respective reservation folders, reservation systems of increasing complexity are implemented.
Short descriptions of the systems are provided at the beginning of the respective main.rs files in
the src folders.

## Example Outputs for Situations with 2 Rooms and 2 Projectors

### Example Output for Reservation System 1
✅: User 1 booked Room from time 1 to time 2.\
✅: User 2 booked Projector from time 1 to time 2.\
✅: User 1 booked Room from time 2 to time 4.\
✅: User 2 booked Projector from time 1 to time 3.\
✅: User 1 booked Room from time 1 to time 2.\
❌: User 3 couldn't book Room from time 1 to time 2 - capacity exceeded.\
❌: User 3 couldn't book Projector from time 1 to time 5 - capacity exceeded.

### Example Output for Reservation System 2

✅: Non-VIP User 1 booked Room from time 1 to time 2.\
✅: Non-VIP User 2 booked Projector from time 1 to time 2.\
✅: VIP User 5 booked Room from time 1 to time 2.\
✅: Non-VIP User 2 booked Projector from time 1 to time 2.\
❌: User 1's booking of facility Room from time 1 to time 2 was cancelled as of a vip booking.\
❌: User 2's booking of facility Projector from time 1 to time 2 was cancelled as of a vip booking.\
✅: VIP User 3 booked Room from time 1 to time 2.\
✅: VIP User 5 booked Projector from time 1 to time 2.\
❌: Non-VIP User 2 received cancellation message.\
✅: VIP User 4 booked Room from time 1 to time 2.\
❌: User 2's booking of facility Projector from time 1 to time 2 was cancelled as of a vip booking.\
❌: Non-VIP User 1 couldn't book Room from time 1 to time 2 - capacity exceeded.\
✅: VIP User 3 booked Projector from time 1 to time 2.\
❌: Non-VIP User 2 received cancellation message.\
❌: Non-VIP User 1 received cancellation message.\
❌: VIP User 4 couldn't book Projector from time 1 to time 2 - capacity exceeded.

### Example Output for Reservation System 3

✅: Non-VIP User 1 booked facility Room from time 1 to time 2.\
✅: Non-VIP User 1 booked facility Projector from time 1 to time 2.\
✅: Non-VIP User 1 successfully booked all facilities.\
✅: Non-VIP User 2 booked facility Room from time 1 to time 2.\
✅: Non-VIP User 2 booked facility Projector from time 1 to time 2.\
✅: Non-VIP User 2 successfully booked all facilities.\
❌: Non-VIP User 1's booking of facility Room from time 1 to time 2 was cancelled as of a vip booking.\
❌: Non-VIP User 1's booking of facility Projector from time 1 to time 2 was cancelled as of a vip booking.\
✅: VIP User 3 booked facility Room from time 1 to time 2.\
✅: VIP User 3 successfully booked all facilities.\
❌: Non-VIP User 1 received cancellation message.\
❌: Non-VIP User 1 received cancellation message.