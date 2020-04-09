import React from 'react';

import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';


function ReleaseHighlightsListPage(props) {
  return (
    <Layout title="Releases" description="Vector releases, highlights, and changelog.">
      <header className="hero hero--clean">
        <div className="container">
          <h1>Vector Releases</h1>
          <div className="hero--subtitle">
            Thoughtful releases to help you get the most out of Vector. Created and curated by the <Link to="/community#team">Vector team</Link>.
          </div>
        </div>
      </header>
      <main className="container container--s">
        Highlights go here
      </main>
    </Layout>
  );
}

export default ReleaseHighlightsListPage;
