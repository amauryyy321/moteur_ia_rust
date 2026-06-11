import { Activity, CircleEqual, Crown, TimerReset } from "lucide-react";
import type { GameStateDto, GameStatus } from "../type";

interface GameStatusProps {
  game: GameStateDto;
}

function statusText(status: GameStatus): string {
  switch (status) {
    case "EnCours":
      return "En cours";
    case "Mat":
      return "Echec et mat";
    case "Pat":
      return "Pat";
    case "Nulle50Coups":
      return "Nulle 50 coups";
    case "NulleRepetition":
      return "Nulle repetition";
  }
}

function StatusIcon({ status }: { status: GameStatus }) {
  if (status === "Mat") {
    return <Crown size={18} />;
  }
  if (status === "Pat" || status === "NulleRepetition") {
    return <CircleEqual size={18} />;
  }
  if (status === "Nulle50Coups") {
    return <TimerReset size={18} />;
  }

  return <Activity size={18} />;
}

export function GameStatus({ game }: GameStatusProps) {
  return (
    <section className="panel status-panel">
      <div className="status-title">
        <StatusIcon status={game.status} />
        <span>{statusText(game.status)}</span>
      </div>
      <div className="stats-grid">
        <div>
          <span>Trait</span>
          <strong>{game.side_to_move}</strong>
        </div>
        <div>
          <span>Coup</span>
          <strong>{game.fullmove_number}</strong>
        </div>
        <div>
          <span>50 coups</span>
          <strong>{game.halfmove_clock}</strong>
        </div>
        <div>
          <span>Legaux</span>
          <strong>{game.legal_moves.length}</strong>
        </div>
      </div>
    </section>
  );
}
