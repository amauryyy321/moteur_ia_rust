import { Circle, CircleDot } from "lucide-react";
import type { Color } from "../type";

interface SideChoiceProps {
  busy: boolean;
  onChoose: (color: Color) => void;
}

export function SideChoice({ busy, onChoose }: SideChoiceProps) {
  return (
    <section className="side-choice panel">
      <div>
        <h2>Choisir votre camp</h2>
        <p>Les blancs commencent. Si vous prenez les noirs, l'IA joue le premier coup.</p>
      </div>
      <div className="side-choice-actions">
        <button type="button" onClick={() => onChoose("Blanc")} disabled={busy}>
          <Circle size={20} />
          <span>Jouer les blancs</span>
        </button>
        <button type="button" onClick={() => onChoose("Noir")} disabled={busy}>
          <CircleDot size={20} />
          <span>Jouer les noirs</span>
        </button>
      </div>
    </section>
  );
}
