interface MoveListProps {
  moves: string[];
}

export function MoveList({ moves }: MoveListProps) {
  return (
    <section className="panel move-panel">
      <div className="panel-heading">
        <h2>Coups</h2>
        <span>{moves.length}</span>
      </div>
      <ol className="move-list">
        {moves.map((move, index) => (
          <li key={`${move}-${index}`}>
            <span>{Math.floor(index / 2) + 1}{index % 2 === 0 ? "." : "..."}</span>
            <strong>{move}</strong>
          </li>
        ))}
      </ol>
    </section>
  );
}
