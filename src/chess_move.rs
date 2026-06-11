use crate::board::Pieces;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Quiet,
    Capture,
    DoublePawnPush,
    EnPassant,
    Castling,
    Promotion,
    PromotionCapture,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub piece: Pieces,
    pub captured: Option<Pieces>,
    pub promotion: Option<Pieces>,
    pub flag: MoveFlag,
}
