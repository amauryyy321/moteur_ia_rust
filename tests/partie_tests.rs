use moteur_ia::partie::{EtatPartie, Partie};

#[test]
pub fn test_nulle_50_coups() {
    let mut partie = Partie::depuis_fen("7k/8/8/8/8/8/8/K7 w - - 100 80").unwrap();

    assert_eq!(partie.etat(), EtatPartie::Nulle50Coups);
}

#[test]
pub fn test_position_mat() {
    let mut partie = Partie::depuis_fen("7k/7Q/6K1/8/8/8/8/8 b - - 0 1").unwrap();

    assert_eq!(partie.etat(), EtatPartie::Mat);
}

#[test]
pub fn test_position_pat() {
    let mut partie = Partie::depuis_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();

    assert_eq!(partie.etat(), EtatPartie::Pat);
}