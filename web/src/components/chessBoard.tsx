import type { Color, GameStateDto, MoveDto } from "../type";
import { Square } from "./square";

interface ChessBoardProps {
  game: GameStateDto;
  orientation: Color;
  selected: string | null;
  onSquareClick: (coord: string) => void;
}

function coordFromIndex(index: number): string {
  const file = index % 8;
  const rank = Math.floor(index / 8);

  return `${String.fromCharCode(97 + file)}${rank + 1}`;
}

function isLastMoveSquare(move: string | undefined, coord: string): boolean {
  if (!move || move.length < 4) {
    return false;
  }

  return move.slice(0, 2) === coord || move.slice(2, 4) === coord;
}

function range(start: number, end: number, step: number): number[] {
  const values = [];

  for (let value = start; step > 0 ? value <= end : value >= end; value += step) {
    values.push(value);
  }

  return values;
}

export function ChessBoard({ game, orientation, selected, onSquareClick }: ChessBoardProps) {
  const legalTargets = new Set(
    game.legal_moves
      .filter((move: MoveDto) => move.from_coord === selected)
      .map((move: MoveDto) => move.to_coord),
  );
  const lastMove = game.move_history[game.move_history.length - 1];
  const ranks = orientation === "Blanc" ? range(7, 0, -1) : range(0, 7, 1);
  const files = orientation === "Blanc" ? range(0, 7, 1) : range(7, 0, -1);
  const squares = [];

  for (let displayRank = 0; displayRank < ranks.length; displayRank += 1) {
    for (let displayFile = 0; displayFile < files.length; displayFile += 1) {
      const rank = ranks[displayRank];
      const file = files[displayFile];
      const index = rank * 8 + file;
      const coord = coordFromIndex(index);

      squares.push(
        <Square
          key={coord}
          coord={coord}
          file={file}
          rank={rank}
          piece={game.board[index]}
          selected={selected === coord}
          legalTarget={legalTargets.has(coord)}
          lastMove={isLastMoveSquare(lastMove, coord)}
          showRankLabel={displayFile === 0}
          showFileLabel={displayRank === 7}
          onClick={() => onSquareClick(coord)}
        />,
      );
    }
  }

  return (
    <div className="board" aria-label="Echiquier">
      {squares}
    </div>
  );
}
