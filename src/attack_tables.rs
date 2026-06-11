/*
les erreurs :
ne pas utiliser move c est un mot cle en rust
faute d inattention sur les vecteurs
1 << position il faut preciser 1u64 << position
*/
use crate::board::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackTables {
    pub knight_attacks: [u64; 64],
    pub king_attacks: [u64; 64],
    pub pawn_attacks: [[u64; 64]; 2],
}

pub fn init_attack_tables() -> AttackTables {
    AttackTables {
        knight_attacks: initialisation_attaque_cavalier(),
        king_attacks: initialisation_attaque_roi(),
        pawn_attacks: initialisation_attaque_pion(),
    }
}

pub fn initialisation_attaque_pion() -> [[u64; 64]; 2] {
    let mut attaque_pion = [[0u64; 64]; 2];
    for i in 0..64 {
        attaque_pion[Color::Blanc as usize][i] = masques_mouvements_pion_blanc(i);
        attaque_pion[Color::Noir as usize][i] = masques_mouvements_pion_noir(i);
    }
    attaque_pion
}
pub fn initialisation_attaque_cavalier() -> [u64; 64] {
    let mut attaque_cavalier = [0u64; 64];
    for i in 0..64 {
        attaque_cavalier[i] = masques_mouvements_cavalier(i);
    }
    attaque_cavalier
}

pub fn initialisation_attaque_roi() -> [u64; 64] {
    let mut attaque_roi = [0u64; 64];
    for i in 0..64 {
        attaque_roi[i] = masques_mouvements_roi(i);
    }

    attaque_roi
}

pub fn masques_mouvements_cavalier(square: usize) -> u64 {
    let file = square % 8;
    let rank = square / 8;
    //39
    let moves = [
        (1, 2),
        (1, -2),
        (-1, 2),
        (-1, -2),
        (2, 1),
        (2, -1),
        (-2, 1),
        (-2, -1),
    ];

    let mut bitboards: u64 = 0;
    for (df, dr) in moves {
        let f = file as i32 + df;
        let r = rank as i32 + dr;
        if f <= 7 && f >= 0 && r <= 7 && r >= 0 {
            let position = (r * 8 + f) as usize;
            bitboards |= 1u64 << position;
        }
    }
    bitboards
}
pub fn masques_mouvements_roi(square: usize) -> u64 {
    let file = square % 8;
    let rank = square / 8;
    //39
    let moves = [
        (1, 0),
        (-1, 0),
        (0, -1),
        (0, 1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    let mut bitboards: u64 = 0;
    for (df, dr) in moves {
        let f = file as i32 + df;
        let r = rank as i32 + dr;
        if f <= 7 && f >= 0 && r <= 7 && r >= 0 {
            let position = (r * 8 + f) as usize;
            bitboards |= 1u64 << position;
        }
    }
    bitboards
}

pub fn masques_mouvements_pion_blanc(square: usize) -> u64 {
    let file = square % 8;
    let rank = square / 8;

    let mouvement_pion_blanc_attaque = [(1, 1), (-1, 1)];

    let mut bitboards: u64 = 0;
    for (df, dr) in mouvement_pion_blanc_attaque {
        let f = file as i32 + df;
        let r = rank as i32 + dr;
        if f <= 7 && f >= 0 && r <= 7 && r >= 0 {
            let position = (r * 8 + f) as usize;
            bitboards |= 1u64 << position;
        }
    }
    bitboards
}

pub fn masques_mouvements_pion_noir(square: usize) -> u64 {
    let file = square % 8;
    let rank = square / 8;

    let mouvement_pion_noir_attaque = [(1, -1), (-1, -1)];

    let mut bitboards: u64 = 0;
    for (df, dr) in mouvement_pion_noir_attaque {
        let f = file as i32 + df;
        let r = rank as i32 + dr;
        if f <= 7 && f >= 0 && r <= 7 && r >= 0 {
            let position = (r * 8 + f) as usize;
            bitboards |= 1u64 << position;
        }
    }
    bitboards
}

pub fn masques_sliding_move(square: usize, occupied: u64, direction: &[(i32, i32)]) -> u64 {
    let file = square % 8;
    let rank = square / 8;

    let mut attaques = 0u64;

    for &(df, dr) in direction {
        let mut f = file as i32 + df;
        let mut r = rank as i32 + dr;
        while (0..8).contains(&r) && (0..8).contains(&f) {
            let position = (r * 8 + f) as usize;
            let target = 1u64 << position;
            attaques |= target;
            //si il y a un obstacle la boucle s arrete
            if occupied & target != 0 {
                break;
            }
            f += df;
            r += dr;
        }
    }

    attaques
}

pub fn masques_mouvements_fou(square: usize, occupied: u64) -> u64 {
    masques_sliding_move(square, occupied, &[(1, -1), (1, 1), (-1, 1), (-1, -1)])
}

pub fn masques_mouvements_tour(square: usize, occupied: u64) -> u64 {
    masques_sliding_move(square, occupied, &[(1, 0), (-1, 0), (0, 1), (0, -1)])
}

pub fn masques_mouvements_dame(square: usize, occupied: u64) -> u64 {
    masques_mouvements_fou(square, occupied) | masques_mouvements_tour(square, occupied)
}
