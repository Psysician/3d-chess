use bevy::math::primitives::InfinitePlane3d;
use bevy::prelude::*;
use chess_core::Square;

// M2 centralizes square/world mapping so board rendering, piece placement, and cursor picking share one coordinate contract.
#[must_use]
pub fn board_axis(index: u8, square_size: f32) -> f32 {
    (f32::from(index) - 3.5) * square_size
}

#[must_use]
pub fn square_to_board_translation(square: Square, square_size: f32, board_height: f32) -> Vec3 {
    Vec3::new(
        board_axis(square.file(), square_size),
        board_height * 0.5,
        board_axis(square.rank(), square_size),
    )
}

#[must_use]
pub fn world_to_square(world: Vec3, square_size: f32) -> Option<Square> {
    let half_board = square_size * 4.0;
    if !(-half_board..half_board).contains(&world.x)
        || !(-half_board..half_board).contains(&world.z)
    {
        return None;
    }

    let file = ((world.x + half_board) / square_size).floor() as i32;
    let rank = ((world.z + half_board) / square_size).floor() as i32;
    if !(0..8).contains(&file) || !(0..8).contains(&rank) {
        return None;
    }

    Some(Square::from_coords_unchecked(file as u8, rank as u8))
}

pub fn board_plane_intersection(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    cursor_position: Vec2,
    board_height: f32,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;
    let distance = ray.intersect_plane(
        Vec3::new(0.0, board_height * 0.5, 0.0),
        InfinitePlane3d::new(Vec3::Y),
    )?;

    Some(ray.get_point(distance))
}

#[cfg(test)]
mod tests {
    use super::{square_to_board_translation, world_to_square};
    use bevy::prelude::Vec3;
    use chess_core::Square;

    #[test]
    fn square_world_mapping_roundtrips_and_rejects_out_of_bounds() {
        let square_size = 1.05;
        let board_height = 0.16;

        for square in [
            Square::from_algebraic("a1").expect("valid square"),
            Square::from_algebraic("d4").expect("valid square"),
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
}
