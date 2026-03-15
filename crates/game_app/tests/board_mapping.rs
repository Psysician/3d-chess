use bevy::prelude::Vec3;
use chess_core::Square;
use game_app::board_coords::{square_to_board_translation, world_to_square};

#[test]
fn board_mapping_roundtrips_centers_and_rejects_out_of_bounds() {
    let square_size = 1.05;
    let board_height = 0.16;

    for square in [
        Square::from_algebraic("a1").expect("valid square"),
        Square::from_algebraic("e4").expect("valid square"),
        Square::from_algebraic("h8").expect("valid square"),
    ] {
        let translation = square_to_board_translation(square, square_size, board_height);
        assert_eq!(world_to_square(translation, square_size), Some(square));
    }

    assert_eq!(
        world_to_square(Vec3::new(square_size * 4.1, 0.0, 0.0), square_size),
        None
    );
    assert_eq!(
        world_to_square(Vec3::new(0.0, 0.0, -square_size * 4.1), square_size),
        None
    );
}
