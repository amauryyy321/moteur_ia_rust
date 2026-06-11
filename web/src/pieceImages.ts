import type { PieceDto } from "./type";

const pieceImages: Record<string, string> = {
  P: "wp",
  N: "wn",
  B: "wb",
  R: "wr",
  Q: "wq",
  K: "wk",
  p: "bp",
  n: "bn",
  b: "bb",
  r: "br",
  q: "bq",
  k: "bk",
};

export function pieceImageSrc(piece: PieceDto): string {
  return `/pieces/standard/${pieceImages[piece.code]}.png`;
}
