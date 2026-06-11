use crate::board::Pieces;
use crate::chess_move::Move;
pub fn square_to_coord(square_bb: u64) -> String {
    let square = square_bb.trailing_zeros() as usize;
    assert!(
        square_bb.count_ones() == 1,
        "plusieurs piece sont sur l echiquier square to coord : {}",
        square
    );

    let rank = (square / 8) + 1;
    let file = (b'a' + (square % 8) as u8) as char;
    format!("{}{}", file, rank)
}

pub fn coord_to_square(coord: &str) -> u64 {
    let bytes = coord.as_bytes();
    let rank = (bytes[1] - b'1') as usize;
    let file = (bytes[0] - b'a') as usize;
    assert!(
        file < 8,
        "l indice file est trop haut dans coord_to_square : {}",
        file
    );
    let square = rank * 8 + file;
    assert!(
        square < 64,
        "l indice square est trop haut dans coord_to_square : {}",
        square
    );
    1u64 << square
}

pub fn move_to_coord(mv: &Move) -> String {
    let promotion_letter = if let Some(promotion_piece) = mv.promotion {
        match (promotion_piece) {
            Pieces::FouBlanc | Pieces::FouNoir => "b",
            Pieces::CavalierBlanc | Pieces::CavalierNoir => "n",
            Pieces::TourNoire | Pieces::TourBlanche => "r",
            Pieces::DameBlanche | Pieces::DameNoire => "q",
            _ => "",
        }
    } else {
        ""
    };
    format!(
        "{}{}{}",
        square_to_coord(1u64 << mv.from),
        square_to_coord(1u64 << mv.to),
        promotion_letter
    )
}

pub fn coord_to_square_index(coord: &str) -> Result<u8, String> {
    if coord.len() != 2 {
        return Err("coord invalide".to_string());
    }

    let bytes = coord.as_bytes();

    let file = match bytes[0] {
        b'a'..=b'h' => bytes[0] - b'a',
        _ => return Err("file invalide".to_string()),
    };

    let rank = match bytes[1] {
        b'1'..=b'8' => bytes[1] - b'1',
        _ => return Err("rank invalide".to_string()),
    };

    Ok(rank * 8 + file)
}

#[test]
#[should_panic]
pub fn test_square_to_coord_many_pieces_1() {
    square_to_coord(1u64 << 5 | 1u64 << 6);
}
#[test]
#[should_panic]
pub fn test_square_to_coord_many_pieces_2() {
    square_to_coord(1u64 << 63 | 1u64 << 4);
}
#[test]
pub fn test_square_to_coord() {
    assert_eq!(square_to_coord(1u64 << 63), "h8");
    assert_eq!(square_to_coord(1u64 << 0), "a1");
}

#[test]
#[should_panic]
pub fn invalid_caracter_test_coord_to_coord() {
    coord_to_square(":;");
}
#[test]
#[should_panic]
pub fn invalid_unicode_1_test_coord_to_coord() {
    coord_to_square("þç");
}
#[test]
#[should_panic]
pub fn invalid_unicode_2_test_coord_to_coord() {
    coord_to_square("øÂ");
}

#[test]
#[should_panic]
pub fn invalid_coordonate_test_coord_1_to_coord() {
    coord_to_square("z4");
}
#[test]
#[should_panic]
pub fn invalid_coordonate_test_coord_2_to_coord() {
    coord_to_square("a9");
}
#[test]
#[should_panic]
pub fn invalid_coordonate_test_coord_3_to_coord() {
    coord_to_square("aa");
}
#[test]
#[should_panic]
pub fn invalid_coordonate_test_coord_4_to_coord() {
    coord_to_square("22");
}
#[test]
pub fn test_coord_to_square() {
    assert_eq!(coord_to_square("h8"), 1u64 << 63);
    assert_eq!(coord_to_square("e4"), 1u64 << 28);
    assert_eq!(coord_to_square("a1"), 1u64 << 0);
}
