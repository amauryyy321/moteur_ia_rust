import type { PieceDto } from "../type";
import { pieceImageSrc } from "../pieceImages";

interface SquareProps {
  coord: string;
  file: number;
  rank: number;
  piece: PieceDto | null;
  selected: boolean;
  legalTarget: boolean;
  lastMove: boolean;
  showRankLabel: boolean;
  showFileLabel: boolean;
  onClick: () => void;
}

export function Square({
  coord,
  file,
  rank,
  piece,
  selected,
  legalTarget,
  lastMove,
  showRankLabel,
  showFileLabel,
  onClick,
}: SquareProps) {
  const dark = (file + rank) % 2 === 0;
  const className = [
    "square",
    dark ? "square-dark" : "square-light",
    selected ? "square-selected" : "",
    legalTarget ? "square-legal" : "",
    lastMove ? "square-last" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <button className={className} type="button" onClick={onClick} aria-label={coord}>
      {showRankLabel && <span className="rank-label">{rank + 1}</span>}
      {showFileLabel && <span className="file-label">{coord[0]}</span>}
      {piece && (
        <img
          className="piece-image"
          src={pieceImageSrc(piece)}
          alt={`${piece.name} ${piece.color}`}
          draggable={false}
        />
      )}
    </button>
  );
}
