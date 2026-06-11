use crate::board::CBoard;
use crate::chess_move::{Move, MoveFlag};
pub fn affichage_echiquier() {
    let mut increment = 63;
    loop {
        increment -= 7;
        if increment < 0 {
            break;
        }
        for _ in 0..7 {
            print!("{}\t", increment);
            increment += 1;
        }
        println!("{}\t", increment);
        increment -= 8;
    }
}

pub fn affichage_position_complete(board: &CBoard) {
    for i in (0..8).rev() {
        for j in 0..8 {
            let case = i * 8 + j;
            let mut verif = 0;
            let mask = 1u64 << case;
            for i in (0..12) {
                if board.piece_bb[i] & mask != 0 {
                    print!("{}\t", i + 1);
                    verif = 1;
                }
            }
            if verif == 0 {
                print!("0\t");
            }
        }
        println!("");
    }
    println!("");
}
pub fn affichage_position_piece(piece_choisis: u64) {
    println!("");
    for i in (0..8).rev() {
        for j in 0..8 {
            let case = i * 8 + j;
            let mask = 1u64 << case;
            if mask & piece_choisis != 0 {
                print!("1\t");
            } else {
                print!("0\t");
            }
        }
        println!("");
    }
    println!("");
}
pub fn affichage_mouvements(mouvements: &Vec<Move>) {
    for (i, coup) in mouvements.iter().enumerate() {
        println!(
            "position origine : {} | position finale : {} , index : {}",
            coup.from, coup.to, i
        );
    }
}
