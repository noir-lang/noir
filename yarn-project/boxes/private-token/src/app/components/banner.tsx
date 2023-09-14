interface Props {
  background: string;
  direction: string;
  animated?: boolean;
}

/**
 *
 * @param background - background color, either "black" or "purple"
 * @returns a moving banner repeating the word PRIVACY
 */
export function Banner({ background, direction, animated }: Props) {
  // Determine direction
  const start = !animated ? '' : direction === 'reverse' ? 'animate-marquee' : 'animate-marquee3';
  const end = !animated ? '' : direction === 'reverse' ? 'animate-marquee2' : 'animate-marquee4';

  // Apply relevant color styles
  const containerStyles =
    background === 'black'
      ? `relative flex overflow-x-hidden bg-indigo-950 text-orange-100`
      : `relative flex overflow-x-hidden bg-orange-100 text-indigo-950`;

  return (
    <div className={containerStyles}>
      <div className={`py-2 whitespace-nowrap ${start}`}>
        {/* Generate text elements */}
        {Array.from({ length: 50 }, (_, index) => (
          <span className="mx-4 text-2xl NBInter" key={index}>
            PRIVACY
          </span>
        ))}
      </div>
      <div className={`absolute top-0 py-2 whitespace-nowrap ${end}`}>
        {/* Generate text elements */}
        {Array.from({ length: 50 }, (_, index) => (
          <span className="mx-4 text-2xl NBInter" key={index}>
            PRIVACY
          </span>
        ))}
      </div>
    </div>
  );
}
