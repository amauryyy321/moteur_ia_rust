import { AlertCircle } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { newGame, playAiMove, playMove } from "./api";
import { ChessBoard } from "./components/chessBoard";
import { Controls } from "./components/controls";
import { GameStatus } from "./components/gameStatus";
import { MoveList } from "./components/moveList";
import { SideChoice } from "./components/sideChoice";
import type { Color, GameStateDto, MoveDto, Promotion } from "./type";

function playerHasTurn(game: GameStateDto, playerColor: Color | null): boolean {
  return Boolean(playerColor && game.status === "EnCours" && game.side_to_move === playerColor);
}

function pieceBelongsToPlayer(game: GameStateDto, playerColor: Color | null, coord: string): boolean {
  if (!playerHasTurn(game, playerColor)) {
    return false;
  }

  const move = game.legal_moves.find((legalMove) => legalMove.from_coord === coord);

  return Boolean(move);
}

function turnMessage(
  game: GameStateDto,
  playerColor: Color | null,
  requestBusy: boolean,
  isAiThinking: boolean,
): string {
  if (requestBusy) {
    return "Coup en cours";
  }

  if (isAiThinking) {
    return "L'IA reflechit";
  }

  if (game.status !== "EnCours") {
    return "Partie terminee";
  }

  return playerHasTurn(game, playerColor) ? "A vous de jouer" : "Tour de l'IA";
}

export function App() {
  const [game, setGame] = useState<GameStateDto | null>(null);
  const [playerColor, setPlayerColor] = useState<Color | null>(null);
  const [choosingSide, setChoosingSide] = useState(true);
  const [selected, setSelected] = useState<string | null>(null);
  const [promotion, setPromotion] = useState<Promotion>("q");
  const [error, setError] = useState<string | null>(null);
  const [requestBusy, setRequestBusy] = useState(false);
  const [isAiThinking, setIsAiThinking] = useState(false);
  const aiRequestInFlight = useRef(false);

  const legalMovesFromSelection = useMemo(() => {
    if (!game || !selected) {
      return [];
    }

    return game.legal_moves.filter((move: MoveDto) => move.from_coord === selected);
  }, [game, selected]);

  const canPlayerMove = Boolean(
    game && playerHasTurn(game, playerColor) && !requestBusy && !isAiThinking && !choosingSide,
  );
  const canRequestAiMove = Boolean(
    game &&
      playerColor &&
      game.status === "EnCours" &&
      game.side_to_move !== playerColor &&
      !requestBusy &&
      !isAiThinking,
  );

  async function requestAiMove() {
    if (!game || !canRequestAiMove || aiRequestInFlight.current) {
      return;
    }

    aiRequestInFlight.current = true;
    setIsAiThinking(true);
    setError(null);
    setSelected(null);

    try {
      const response = await playAiMove();
      setGame(response.state);

      if (!response.ok) {
        setError(response.error ?? "Coup IA refuse");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erreur inconnue");
    } finally {
      aiRequestInFlight.current = false;
      setIsAiThinking(false);
    }
  }

  useEffect(() => {
    if (canRequestAiMove) {
      void requestAiMove();
    }
  }, [canRequestAiMove]);

  async function handleChooseSide(color: Color) {
    aiRequestInFlight.current = false;
    setRequestBusy(true);
    setIsAiThinking(color === "Noir");
    setError(null);
    setSelected(null);

    try {
      const state = await newGame(color);
      setPlayerColor(color);
      setGame(state);
      setChoosingSide(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erreur inconnue");
    } finally {
      setRequestBusy(false);
      setIsAiThinking(false);
    }
  }

  async function refreshAfterMove(from: string, to: string) {
    if (!game || !canPlayerMove) {
      return;
    }

    setRequestBusy(true);
    setError(null);

    const promotionMove = legalMovesFromSelection.some(
      (move) => move.to_coord === to && move.promotion !== null,
    );

    try {
      const response = await playMove({
        from,
        to,
        promotion: promotionMove ? promotion : undefined,
      });

      setGame(response.state);
      setSelected(null);

      if (!response.ok) {
        setError(response.error ?? "Coup refuse");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Erreur inconnue");
    } finally {
      setRequestBusy(false);
    }
  }

  function handleSquareClick(coord: string) {
    if (!game || !canPlayerMove) {
      return;
    }

    if (!selected) {
      setSelected(pieceBelongsToPlayer(game, playerColor, coord) ? coord : null);
      return;
    }

    const legalTarget = legalMovesFromSelection.some((move) => move.to_coord === coord);

    if (legalTarget) {
      void refreshAfterMove(selected, coord);
      return;
    }

    setSelected(pieceBelongsToPlayer(game, playerColor, coord) ? coord : null);
  }

  function handleNewGame() {
    aiRequestInFlight.current = false;
    setChoosingSide(true);
    setSelected(null);
    setError(null);
  }

  const busy = requestBusy || isAiThinking;

  if (choosingSide || !game || !playerColor) {
    return (
      <main className="app-shell choice-shell">
        <section className="board-zone intro-zone">
          <header className="top-bar">
            <div>
              <h1>Echecs IA</h1>
              <span>Nouvelle partie</span>
            </div>
            {error && (
              <div className="error-pill" role="alert">
                <AlertCircle size={17} />
                <span>{error}</span>
              </div>
            )}
          </header>
          <div className="board-preview" aria-hidden="true" />
        </section>

        <aside className="side-panel">
          <SideChoice busy={busy} onChoose={handleChooseSide} />
        </aside>
      </main>
    );
  }

  return (
    <main className="app-shell">
      <section className="board-zone">
        <header className="top-bar">
          <div>
            <h1>Echecs IA</h1>
            <span>{turnMessage(game, playerColor, requestBusy, isAiThinking)}</span>
          </div>
          {error && (
            <div className="error-pill" role="alert">
              <AlertCircle size={17} />
              <span>{error}</span>
            </div>
          )}
        </header>
        <ChessBoard
          game={game}
          orientation={playerColor}
          selected={selected}
          onSquareClick={handleSquareClick}
        />
      </section>

      <aside className="side-panel">
        <GameStatus game={game} />
        <Controls
          promotion={promotion}
          onPromotionChange={setPromotion}
          onAiMove={requestAiMove}
          onNewGame={handleNewGame}
          busy={requestBusy}
          aiThinking={isAiThinking}
          canRequestAiMove={canRequestAiMove}
          gameEnded={game.status !== "EnCours"}
        />
        <MoveList moves={game.move_history} />
      </aside>
    </main>
  );
}
