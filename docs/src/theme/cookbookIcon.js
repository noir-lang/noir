import { useColorMode } from '@docusaurus/theme-common';
import { useEffect } from 'react';
export const ColorModeProvider = () => {
  const { colorMode } = useColorMode();

  useEffect(() => {
    if (document.querySelector('ask-cookbook')) {
      if (colorMode === 'dark') {
        const cookbookIcons = document.querySelector('ask-cookbook').shadowRoot.querySelector('button img');
        cookbookIcons.setAttribute('src', '/img/faviconDark.svg');
      } else {
        const cookbookIcons = document.querySelector('ask-cookbook').shadowRoot.querySelector('button img');
        cookbookIcons.setAttribute('src', '/img/favicon.svg');
      }
    }
  }, [colorMode]);

  return '';
};
