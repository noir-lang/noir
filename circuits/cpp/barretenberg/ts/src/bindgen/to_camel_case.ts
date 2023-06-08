export function toCamelCase(input: string): string {
  const words = input.split('_');
  const camelCasedWords = words.map((word, index) => {
    if (index === 0) {
      return word;
    }
    return word.charAt(0).toUpperCase() + word.slice(1);
  });
  return camelCasedWords.join('');
}
