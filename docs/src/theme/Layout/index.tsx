import React, { type ReactNode } from 'react';
import Layout from '@theme-original/Layout';
import type LayoutType from '@theme-original/Layout';
import type { WrapperProps } from '@docusaurus/types';
import { ColorModeProvider } from '../cookbookIcon';

type Props = WrapperProps<typeof LayoutType>;

export default function LayoutWrapper(props: Props): ReactNode {
  return (
    <>
      <Layout {...props}>
        <ColorModeProvider />
        {props.children}
      </Layout>
    </>
  );
}
