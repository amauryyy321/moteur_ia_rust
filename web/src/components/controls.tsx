import { Bot, RotateCcw } from "lucide-react";
import type { Promotion } from "../type";

interface ControlsProps {
  promotion: Promotion;
  onPromotionChange: (promotion: Promotion) => void;
  onAiMove: () => void;
  onNewGame: () => void;
  busy: boolean;
  aiThinking: boolean;
  canRequestAiMove: boolean;
  gameEnded: boolean;
}

const promotions: Array<{ value: Promotion; label: string }> = [
  { value: "q", label: "Dame" },
  { value: "r", label: "Tour" },
  { value: "b", label: "Fou" },
  { value: "n", label: "Cavalier" },
];

export function Controls({
  promotion,
  onPromotionChange,
  onAiMove,
  onNewGame,
  busy,
  aiThinking,
  canRequestAiMove,
  gameEnded,
}: ControlsProps) {
  return (
    <section className="panel controls-panel">
      <div className="promotion-group" role="group" aria-label="Promotion">
        {promotions.map((item) => (
          <button
            key={item.value}
            type="button"
            className={promotion === item.value ? "segmented active" : "segmented"}
            onClick={() => onPromotionChange(item.value)}
          >
            {item.label}
          </button>
        ))}
      </div>
      <button
        className="primary-action"
        type="button"
        onClick={onAiMove}
        disabled={busy || aiThinking || gameEnded || !canRequestAiMove}
      >
        <Bot size={18} />
        <span>{aiThinking ? "IA reflechit" : "Forcer l'IA"}</span>
      </button>
      <button className="primary-action" type="button" onClick={onNewGame} disabled={busy || aiThinking}>
        <RotateCcw size={18} />
        <span>Nouvelle partie</span>
      </button>
    </section>
  );
}
