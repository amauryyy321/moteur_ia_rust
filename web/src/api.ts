import type { ApiResponse, Color, GameStateDto, MoveRequest, NewGameRequest } from "./type";

async function readResponse(response: Response): Promise<ApiResponse> {
  if (!response.ok) {
    throw new Error(`Erreur HTTP ${response.status}`);
  }

  return response.json();
}

export async function fetchGameState(): Promise<GameStateDto> {
  const response = await readResponse(await fetch("/api/state"));

  if (!response.ok && response.error) {
    throw new Error(response.error);
  }

  return response.state;
}

export async function playMove(move: MoveRequest): Promise<ApiResponse> {
  return readResponse(
    await fetch("/api/move", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(move),
    }),
  );
}

export async function playAiMove(): Promise<ApiResponse> {
  return readResponse(
    await fetch("/api/ai-move", {
      method: "POST",
    }),
  );
}

export async function newGame(playerColor: Color): Promise<GameStateDto> {
  const body: NewGameRequest = {
    player_color: playerColor,
  };

  const response = await readResponse(
    await fetch("/api/new", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    }),
  );

  if (!response.ok && response.error) {
    throw new Error(response.error);
  }

  return response.state;
}
