import React from 'react';

import Layout from '@theme/Layout';
import MDXComponents from '@theme/MDXComponents';
import {MDXProvider} from '@mdx-js/react';

import classnames from 'classnames';
import styles from './styles.module.css';

function ReleasePage(props) {
  let title = 'Change me';
  let description = 'Change me';
  const {content: ReleaseContents} = props;

  //
  // Render
  //

  return (
    <Layout title={title} description={`${title}, in minutes, for free`}>
      <header className={`hero domain-bg domain-bg--nodes`}>
        <div className="container">
          <h1 className={styles.header}>{title}</h1>
          <div className="hero--subtitle">{description}</div>
        </div>
      </header>
      <main className={classnames('container', 'container--l', styles.container)}>
        <div className="markdown">
          <MDXProvider components={MDXComponents}><ReleaseContents /></MDXProvider>
        </div>
      </main>
    </Layout>
  );
}

export default ReleasePage;
