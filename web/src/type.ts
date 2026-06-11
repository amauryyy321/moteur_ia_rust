export type Color = "Blanc" | "Noir";

export type GameStatus =
  | "EnCours"
  | "Mat"
  | "Pat"
  | "Nulle50Coups"
  | "NulleRepetition";

export type Promotion = "q" | "r" | "b" | "n";

export interface PieceDto {
  code: string;
  name: string;
  color: Color;
  symbol: string;
}

export interface MoveDto {
  from: number;
  to: number;
  from_coord: string;
  to_coord: string;
  promotion: Promotion | null;
  notation: string;
}

export interface GameStateDto {
  status: GameStatus;
  side_to_move: Color;
  board: Array<PieceDto | null>;
  legal_moves: MoveDto[];
  move_history: string[];
  halfmove_clock: number;
  fullmove_number: number;
}

export interface ApiResponse {
  ok: boolean;
  error: string | null;
  state: GameStateDto;
}

export interface MoveRequest {
  from: string;
  to: string;
  promotion?: Promotion;
}

export interface NewGameRequest {
  player_color: Color;
}
