import React from 'react';

import Avatar from '@site/src/components/Avatar';
import Layout from '@theme/Layout';
import MDXComponents from '@theme/MDXComponents';
import {MDXProvider} from '@mdx-js/react';
import Tags from '@site/src/components/Tags';

import classnames from 'classnames';
import dateFormat from 'dateformat';
import styles from './styles.module.css';

function ReleaseHighlightPage(props) {
  let description = 'Change me';
  const {content: ReleaseHighlightContents} = props;
  const {frontMatter, metadata} = ReleaseHighlightContents;
  const {author_github, id, title} = frontMatter;
  const {date: dateString, tags} = metadata;
  const date = new Date(Date.parse(dateString));

  //
  // Render
  //

  return (
    <Layout title={title} description={`${title}, in minutes, for free`}>
      <article className={styles.blogPost}>
        <header className={classnames('hero', 'domain-bg', 'domain-bg--nodes', styles.header)}>
          <div className={classnames('container', styles.headerContainer)}>
            <Avatar github={author_github} size="lg" nameSuffix={<> / <time pubdate="pubdate" dateTime={date.toISOString()}>{dateFormat(date, "mmm dS")}</time></>} rel="author" subTitle={false} vertical={true} />
            <h1>{title}</h1>
            <div className={styles.headerTags}>
              <Tags colorProfile="blog" tags={tags} />
            </div>
          </div>
        </header>
        <div className="container container--xs margin-vert--xl">
          <section className="markdown align-text-edges dropcap">
            <MDXProvider components={MDXComponents}><ReleaseHighlightContents /></MDXProvider>
          </section>
          <section className={classnames('panel', styles.mailingList)} style={{textAlign: 'center'}}>
            <div className={styles.mailingListTitle}>
              <i className="feather icon-mail"></i> Vector In Your Inbox!
            </div>
            <p>
              One email on the 1st of the month. No spam, ever.
            </p>
          </section>
          {(metadata.nextItem || metadata.prevItem) && (
            <div className="margin-vert--xl">
              <BlogPostPaginator
                nextItem={metadata.nextItem}
                prevItem={metadata.prevItem}
              />
            </div>
          )}
        </div>
      </article>
    </Layout>
  );
}

export default ReleaseHighlightPage;
