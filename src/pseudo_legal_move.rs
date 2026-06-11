//cette fonction va prendre en entree le tableau et ressortir un Vecteur de mouvement
use crate::affichage::affichage_position_piece;
use crate::attack_tables::{
    AttackTables, masques_mouvements_cavalier, masques_mouvements_dame, masques_mouvements_fou,
    masques_mouvements_pion_blanc, masques_mouvements_pion_noir, masques_mouvements_roi,
    masques_mouvements_tour,
};
use crate::board::{
    BLACK_KINGSIDE, BLACK_QUEENSIDE, CBoard, Color, Pieces, WHITE_KINGSIDE, WHITE_QUEENSIDE,
};
use crate::chess_move::{Move, MoveFlag};
use crate::legality::is_square_attacked;

pub fn generate_pseudo_legal_move(board: &CBoard, tables: &AttackTables) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    generer_mouvement_pions(board, &mut moves);
    generer_mouvement_cavaliers(board, tables, &mut moves);
    generer_mouvement_fous(board, tables, &mut moves);
    generer_mouvement_tours(board, tables, &mut moves);
    generer_mouvement_rois(board, tables, &mut moves);
    generer_mouvement_dames(board, tables, &mut moves);

    moves
}
// from case de depart du coup
// to case d arrive du coup
// Piece donne le type de la piece qui bouge
// caprured indique le type de la piece capturer si capture
// promotion indique si un pion est sur la case de promotion
// flag : indique different flag

pub fn piece_on_square(board: &CBoard, square: u8) -> Option<Pieces> {
    let square_bb = 1u64 << square;
    for i in 0..12 {
        if board.piece_bb[i] & square_bb != 0 {
            return Some(match i {
                0 => Pieces::PionBlanc,
                1 => Pieces::PionNoir,
                2 => Pieces::CavalierBlanc,
                3 => Pieces::CavalierNoir,
                4 => Pieces::FouBlanc,
                5 => Pieces::FouNoir,
                6 => Pieces::TourBlanche,
                7 => Pieces::TourNoire,
                8 => Pieces::DameBlanche,
                9 => Pieces::DameNoire,
                10 => Pieces::RoiBlanc,
                11 => Pieces::RoiNoir,
                _ => unreachable!(),
            });
        }
    }
    None
}
pub fn generer_mouvement_pions(board: &CBoard, moves: &mut Vec<Move>) {
    match board.side_to_move {
        Color::Blanc => generer_mouvement_pion_blanc(board, moves),
        Color::Noir => generer_mouvement_pion_noir(board, moves),
    }
}

//inventaire :
//trailing_zeros = donne l’index du premier bit à 1
//pions &= pions - 1; enleve le bit le plus a droite

pub fn generer_mouvement_pion_noir(board: &CBoard, moves: &mut Vec<Move>) {
    let mut pions = board.piece_bb[Pieces::PionNoir as usize];
    while pions != 0 {
        let from = pions.trailing_zeros() as u8;

        //le pion avance de 1
        if from >= 8 {
            let to: u8 = from - 8;
            let promotion_rank = to / 8 == 0;
            if (board.occupe_bb & (1u64 << to)) == 0 {
                if promotion_rank {
                    let promotion = [
                        Pieces::DameNoire,
                        Pieces::TourNoire,
                        Pieces::FouNoir,
                        Pieces::CavalierNoir,
                    ];
                    for promotion_coup in promotion {
                        let latent_move = Move {
                            from,
                            to,
                            piece: Pieces::PionNoir,
                            captured: None,
                            promotion: Some(promotion_coup),
                            flag: MoveFlag::Promotion,
                        };
                        moves.push(latent_move);
                    }
                } else {
                    let latent_move = Move {
                        from,
                        to,
                        piece: Pieces::PionNoir,
                        captured: None,
                        promotion: None,
                        flag: MoveFlag::Quiet,
                    };
                    moves.push(latent_move);
                }
            }
        }

        //le pion avance de 2
        if (from / 8 == 6)
            && (board.occupe_bb & ((1u64 << (from - 8)) | (1u64 << (from - 16)))) == 0
        {
            let latent_move = Move {
                from,
                to: (from - 16),
                piece: Pieces::PionNoir,
                captured: None,
                promotion: None,
                flag: MoveFlag::DoublePawnPush,
            };
            moves.push(latent_move);
        }

        // prise avec le pion a-1 et a+1
        let mut mouvement_possible = board.piece_bb[Pieces::PiecesBlanches as usize]
            & masques_mouvements_pion_noir(from as usize);
        while mouvement_possible != 0 {
            let to = mouvement_possible.trailing_zeros() as u8;
            let promotion_rank = to / 8 == 0;
            let piece_capture = piece_on_square(board, to);

            if promotion_rank {
                let promotion = [
                    Pieces::DameNoire,
                    Pieces::TourNoire,
                    Pieces::FouNoir,
                    Pieces::CavalierNoir,
                ];
                for promotion_coup in promotion {
                    let latent_move = Move {
                        from,
                        to,
                        piece: Pieces::PionNoir,
                        captured: piece_capture,
                        promotion: Some(promotion_coup),
                        flag: MoveFlag::PromotionCapture,
                    };
                    moves.push(latent_move);
                }
            } else {
                let latent_move = Move {
                    from,
                    to,
                    piece: Pieces::PionNoir,
                    captured: piece_capture,
                    promotion: None,
                    flag: MoveFlag::Capture,
                };
                moves.push(latent_move);
            }

            mouvement_possible &= mouvement_possible - 1;
        }

        //prise en passant
        if let Some(eq_square) = board.en_passant_square {
            let attack_table = masques_mouvements_pion_noir(from as usize);
            if (attack_table & (1u64 << eq_square)) != 0 {
                let latent_move = Move {
                    from,
                    to: eq_square,
                    piece: Pieces::PionNoir,
                    captured: Some(Pieces::PionBlanc),
                    promotion: None,
                    flag: MoveFlag::EnPassant,
                };
                moves.push(latent_move);
            }
        }

        pions &= pions - 1;
    }
}
pub fn generer_mouvement_pion_blanc(board: &CBoard, moves: &mut Vec<Move>) {
    let mut pions = board.piece_bb[Pieces::PionBlanc as usize];
    while pions != 0 {
        let from = pions.trailing_zeros() as u8;

        //le pion avance de 1
        if from <= 55 {
            let to: u8 = from + 8;
            let promotion_rank = to / 8 == 7;
            if (board.occupe_bb & (1u64 << to)) == 0 {
                if promotion_rank {
                    let promotion = [
                        Pieces::DameBlanche,
                        Pieces::TourBlanche,
                        Pieces::FouBlanc,
                        Pieces::CavalierBlanc,
                    ];
                    for promotion_coup in promotion {
                        let latent_move = Move {
                            from,
                            to,
                            piece: Pieces::PionBlanc,
                            captured: None,
                            promotion: Some(promotion_coup),
                            flag: MoveFlag::Promotion,
                        };
                        moves.push(latent_move);
                    }
                } else {
                    let latent_move = Move {
                        from,
                        to,
                        piece: Pieces::PionBlanc,
                        captured: None,
                        promotion: None,
                        flag: MoveFlag::Quiet,
                    };
                    moves.push(latent_move);
                }
            }
        }

        //le pion avance de 2
        if (from / 8 == 1)
            && (board.occupe_bb & ((1u64 << (from + 8)) | (1u64 << (from + 16)))) == 0
        {
            let latent_move = Move {
                from,
                to: (from + 16),
                piece: Pieces::PionBlanc,
                captured: None,
                promotion: None,
                flag: MoveFlag::DoublePawnPush,
            };
            moves.push(latent_move);
        }

        // prise avec le pion a-1 et a+1
        let mut mouvement_possible = board.piece_bb[Pieces::PiecesNoires as usize]
            & masques_mouvements_pion_blanc(from as usize);
        while mouvement_possible != 0 {
            let to = mouvement_possible.trailing_zeros() as u8;
            let promotion_rank = to / 8 == 7;
            let piece_capture = piece_on_square(board, to);

            if promotion_rank {
                let promotion = [
                    Pieces::DameBlanche,
                    Pieces::TourBlanche,
                    Pieces::FouBlanc,
                    Pieces::CavalierBlanc,
                ];
                for promotion_coup in promotion {
                    let latent_move = Move {
                        from,
                        to,
                        piece: Pieces::PionBlanc,
                        captured: piece_capture,
                        promotion: Some(promotion_coup),
                        flag: MoveFlag::PromotionCapture,
                    };
                    moves.push(latent_move);
                }
            } else {
                let latent_move = Move {
                    from,
                    to,
                    piece: Pieces::PionBlanc,
                    captured: piece_capture,
                    promotion: None,
                    flag: MoveFlag::Capture,
                };
                moves.push(latent_move);
            }

            mouvement_possible &= mouvement_possible - 1;
        }

        //prise en passant
        if let Some(eq_square) = board.en_passant_square {
            let attack_table = masques_mouvements_pion_blanc(from as usize);
            if (attack_table & (1u64 << eq_square)) != 0 {
                let latent_move = Move {
                    from,
                    to: eq_square,
                    piece: Pieces::PionBlanc,
                    captured: Some(Pieces::PionNoir),
                    promotion: None,
                    flag: MoveFlag::EnPassant,
                };
                moves.push(latent_move);
            }
        }

        pions &= pions - 1;
    }
    //a faire et a verifier
}
pub fn generer_mouvement_cavaliers(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    match board.side_to_move {
        Color::Blanc => generer_mouvement_cavalier_blanc(board, tables, moves),
        Color::Noir => generer_mouvement_cavalier_noir(board, tables, moves),
    }
}
pub fn generer_mouvement_cavalier_noir(
    board: &CBoard,
    tables: &AttackTables,
    moves: &mut Vec<Move>,
) {
    let mut cavalier = board.piece_bb[Pieces::CavalierNoir as usize];
    while cavalier != 0 {
        let from = cavalier.trailing_zeros() as u8;
        let mut possibilities = masques_mouvements_cavalier(from as usize)
            & !(board.piece_bb[Pieces::PiecesNoires as usize]);

        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture =
                ((board.piece_bb[Pieces::PiecesBlanches as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };

            let latent_move = Move {
                from,
                to,
                piece: Pieces::CavalierNoir,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }
        cavalier &= cavalier - 1;
    }
}
pub fn generer_mouvement_cavalier_blanc(
    board: &CBoard,
    tables: &AttackTables,
    moves: &mut Vec<Move>,
) {
    let mut cavalier = board.piece_bb[Pieces::CavalierBlanc as usize];
    while cavalier != 0 {
        let from = cavalier.trailing_zeros() as u8;
        let mut possibilities = masques_mouvements_cavalier(from as usize)
            & !(board.piece_bb[Pieces::PiecesBlanches as usize]);

        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture = ((board.piece_bb[Pieces::PiecesNoires as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };

            let latent_move = Move {
                from,
                to,
                piece: Pieces::CavalierBlanc,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }

        cavalier &= cavalier - 1;
    }
}

pub fn generer_mouvement_rois(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    match board.side_to_move {
        Color::Blanc => {
            generer_mouvement_roi_blanc(board, tables, moves);
            generer_roque_blanc(board, tables, moves);
        }
        Color::Noir => {
            generer_mouvement_roi_noir(board, tables, moves);
            generer_roque_noir(board, tables, moves);
        }
    }
}

pub fn generer_roque_blanc(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    if board.castling_rights & WHITE_KINGSIDE != 0 {
        let case_vide = board.occupe_bb & ((1u64 << 5) | (1u64 << 6));
        let case_non_attaquer = !is_square_attacked(board, tables, 4, Color::Noir)
            && !is_square_attacked(board, tables, 5, Color::Noir)
            && !is_square_attacked(board, tables, 6, Color::Noir);
        if case_non_attaquer && case_vide == 0 {
            moves.push(Move {
                from: 4,
                to: 6,
                piece: Pieces::RoiBlanc,
                captured: None,
                promotion: None,
                flag: MoveFlag::Castling,
            });
        }
    }
    if board.castling_rights & WHITE_QUEENSIDE != 0 {
        let case_vide = board.occupe_bb & ((1u64 << 3) | (1u64 << 2) | (1u64 << 1));
        let case_non_attaquer = !is_square_attacked(board, tables, 4, Color::Noir)
            && !is_square_attacked(board, tables, 3, Color::Noir)
            && !is_square_attacked(board, tables, 2, Color::Noir);
        if case_non_attaquer && case_vide == 0 {
            moves.push(Move {
                from: 4,
                to: 2,
                piece: Pieces::RoiBlanc,
                captured: None,
                promotion: None,
                flag: MoveFlag::Castling,
            });
        }
    }
}
pub fn generer_roque_noir(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    if board.castling_rights & BLACK_KINGSIDE != 0 {
        let case_vide = board.occupe_bb & ((1u64 << 61) | (1u64 << 62));
        let case_non_attaquer = !is_square_attacked(board, tables, 60, Color::Blanc)
            && !is_square_attacked(board, tables, 61, Color::Blanc)
            && !is_square_attacked(board, tables, 62, Color::Blanc);
        if case_non_attaquer && case_vide == 0 {
            moves.push(Move {
                from: 60,
                to: 62,
                piece: Pieces::RoiNoir,
                captured: None,
                promotion: None,
                flag: MoveFlag::Castling,
            });
        }
    }
    if board.castling_rights & BLACK_QUEENSIDE != 0 {
        let case_vide = board.occupe_bb & ((1u64 << 57) | (1u64 << 58) | (1u64 << 59));
        let case_non_attaquer = !is_square_attacked(board, tables, 58, Color::Blanc)
            && !is_square_attacked(board, tables, 59, Color::Blanc)
            && !is_square_attacked(board, tables, 60, Color::Blanc);
        if case_non_attaquer && case_vide == 0 {
            moves.push(Move {
                from: 60,
                to: 58,
                piece: Pieces::RoiNoir,
                captured: None,
                promotion: None,
                flag: MoveFlag::Castling,
            });
        }
    }
}

pub fn generer_mouvement_roi_blanc(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    let roi = board.piece_bb[Pieces::RoiBlanc as usize];
    let from = roi.trailing_zeros() as u8;
    let mut possibilities =
        masques_mouvements_roi(from as usize) & !(board.piece_bb[Pieces::PiecesBlanches as usize]);

    while possibilities != 0 {
        //on resupere les mouvement possibles jusqu a qu il y en ai plus dans le masques
        let to = possibilities.trailing_zeros() as u8;

        let is_capture = ((board.piece_bb[Pieces::PiecesNoires as usize]) & (1u64 << to)) != 0;
        let piece_capture = if is_capture {
            piece_on_square(board, to)
        } else {
            None
        };

        let flag = if is_capture {
            MoveFlag::Capture
        } else {
            MoveFlag::Quiet
        };

        let latent_move = Move {
            from,
            to,
            piece: Pieces::RoiBlanc,
            captured: piece_capture,
            promotion: None,
            flag,
        };
        moves.push(latent_move);
        possibilities &= possibilities - 1;
    }
}
pub fn generer_mouvement_roi_noir(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    // a faire plus verifier avec l ia que on fait pas des betises
    let roi = board.piece_bb[Pieces::RoiNoir as usize];
    let from = roi.trailing_zeros() as u8;
    let mut possibilities =
        masques_mouvements_roi(from as usize) & !(board.piece_bb[Pieces::PiecesNoires as usize]);

    while possibilities != 0 {
        let to = possibilities.trailing_zeros() as u8;

        let is_capture = ((board.piece_bb[Pieces::PiecesBlanches as usize]) & (1u64 << to)) != 0;
        let piece_capture = if is_capture {
            piece_on_square(board, to)
        } else {
            None
        };

        let flag = if is_capture {
            MoveFlag::Capture
        } else {
            MoveFlag::Quiet
        };

        let latent_move = Move {
            from,
            to,
            piece: Pieces::RoiNoir,
            captured: piece_capture,
            promotion: None,
            flag,
        };
        moves.push(latent_move);
        possibilities &= possibilities - 1;
    }
}

pub fn generer_mouvement_tours(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    match board.side_to_move {
        Color::Blanc => generer_mouvement_tour_blanche(board, tables, moves),
        Color::Noir => generer_mouvement_tour_noire(board, tables, moves),
    }
}
pub fn generer_mouvement_tour_noire(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    let mut tour = board.piece_bb[Pieces::TourNoire as usize];
    while tour != 0 {
        let from = tour.trailing_zeros() as u8;
        let mut possibilities = masques_mouvements_tour(from as usize, board.occupe_bb)
            & !(board.piece_bb[Pieces::PiecesNoires as usize]);
        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture =
                ((board.piece_bb[Pieces::PiecesBlanches as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };

            let latent_move = Move {
                from,
                to,
                piece: Pieces::TourNoire,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }
        tour &= tour - 1;
    }
}
pub fn generer_mouvement_tour_blanche(
    board: &CBoard,
    tables: &AttackTables,
    moves: &mut Vec<Move>,
) {
    let mut tour = board.piece_bb[Pieces::TourBlanche as usize];
    while tour != 0 {
        let from = tour.trailing_zeros() as u8;

        let mut possibilities = masques_mouvements_tour(from as usize, board.occupe_bb)
            & !(board.piece_bb[Pieces::PiecesBlanches as usize]);
        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture = ((board.piece_bb[Pieces::PiecesNoires as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };

            let latent_move = Move {
                from,
                to,
                piece: Pieces::TourBlanche,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }
        tour &= tour - 1;
    }
}

pub fn generer_mouvement_fous(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    match board.side_to_move {
        Color::Blanc => generer_mouvement_fou_blanc(board, tables, moves),
        Color::Noir => generer_mouvement_fou_noir(board, tables, moves),
    }
}
pub fn generer_mouvement_fou_noir(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    let mut fou = board.piece_bb[Pieces::FouNoir as usize];
    while fou != 0 {
        let from = fou.trailing_zeros() as u8;
        let mut possibilities = masques_mouvements_fou(from as usize, board.occupe_bb)
            & !(board.piece_bb[Pieces::PiecesNoires as usize]);
        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture =
                ((board.piece_bb[Pieces::PiecesBlanches as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };
            let latent_move = Move {
                from,
                to,
                piece: Pieces::FouNoir,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }
        fou &= fou - 1;
    }
}
pub fn generer_mouvement_fou_blanc(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    let mut fou = board.piece_bb[Pieces::FouBlanc as usize];
    while fou != 0 {
        let from = fou.trailing_zeros() as u8;
        let mut possibilities = masques_mouvements_fou(from as usize, board.occupe_bb)
            & !(board.piece_bb[Pieces::PiecesBlanches as usize]);
        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture = ((board.piece_bb[Pieces::PiecesNoires as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };

            let latent_move = Move {
                from,
                to,
                piece: Pieces::FouBlanc,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }
        fou &= fou - 1;
    }
}

pub fn generer_mouvement_dames(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    match board.side_to_move {
        Color::Blanc => generer_mouvement_dame_blanche(board, tables, moves),
        Color::Noir => generer_mouvement_dame_noire(board, tables, moves),
    }
}
pub fn generer_mouvement_dame_noire(board: &CBoard, tables: &AttackTables, moves: &mut Vec<Move>) {
    let mut dame = board.piece_bb[Pieces::DameNoire as usize];

    while dame != 0 {
        let from = dame.trailing_zeros() as u8;
        let mut possibilities = masques_mouvements_dame(from as usize, board.occupe_bb)
            & !(board.piece_bb[Pieces::PiecesNoires as usize]);
        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture =
                ((board.piece_bb[Pieces::PiecesBlanches as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };

            let latent_move = Move {
                from,
                to,
                piece: Pieces::DameNoire,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }
        dame &= dame - 1;
    }
}
pub fn generer_mouvement_dame_blanche(
    board: &CBoard,
    tables: &AttackTables,
    moves: &mut Vec<Move>,
) {
    let mut dame = board.piece_bb[Pieces::DameBlanche as usize];

    while dame != 0 {
        let from = dame.trailing_zeros() as u8;
        let mut possibilities = masques_mouvements_dame(from as usize, board.occupe_bb)
            & !(board.piece_bb[Pieces::PiecesBlanches as usize]);
        while possibilities != 0 {
            let to = possibilities.trailing_zeros() as u8;

            let is_capture = ((board.piece_bb[Pieces::PiecesNoires as usize]) & (1u64 << to)) != 0;
            let piece_capture = if is_capture {
                piece_on_square(board, to)
            } else {
                None
            };

            let flag = if is_capture {
                MoveFlag::Capture
            } else {
                MoveFlag::Quiet
            };

            let latent_move = Move {
                from,
                to,
                piece: Pieces::DameBlanche,
                captured: piece_capture,
                promotion: None,
                flag,
            };
            moves.push(latent_move);
            possibilities &= possibilities - 1;
        }
        dame &= dame - 1;
    }
}
