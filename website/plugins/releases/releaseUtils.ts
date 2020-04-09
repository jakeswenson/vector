import {PluginOptions, Release, ReleaseHighlight} from './types';
import {LoadContext} from '@docusaurus/types';

import _ from 'lodash';
import fs from 'fs-extra';
import globby from 'globby';
import path from 'path';
import {parse, normalizeUrl, aliasedSitePath} from '@docusaurus/utils';
import readingTime from 'reading-time';

export function truncate(fileString: string, truncateMarker: RegExp) {
  return fileString.split(truncateMarker, 1).shift()!;
}

export async function generateHighlights(
  releaseDir: string,
  {siteConfig, siteDir}: LoadContext,
  options: PluginOptions,
) {
  const {include, routeBasePath, truncateMarker} = options;

  if (!fs.existsSync(releaseDir)) {
    return [];
  }

  const {baseUrl = ''} = siteConfig;
  const highlightInclude = include.map(glob => `highlights/${glob}`);
  const highlightFiles = await globby(highlightInclude, {
    cwd: releaseDir,
  });

  const releaseHighlights: ReleaseHighlight[] = [];

  await Promise.all(
    highlightFiles.map(async (relativeSource: string) => {
      const source = path.join(releaseDir, relativeSource);
      const aliasedSource = aliasedSitePath(source, siteDir);
      const fileString = await fs.readFile(source, 'utf-8');
      const readingStats = readingTime(fileString);
      const {frontMatter, content, excerpt} = parse(fileString);

      if (frontMatter.draft && process.env.NODE_ENV === 'production') {
        return;
      }

      let date = frontMatter.date;
      let linkName = relativeSource.replace(/\.mdx?$/, '');
      let seriesPosition = frontMatter.series_position;
      let tags = frontMatter.tags || [];
      let title = frontMatter.title || linkName;

      releaseHighlights.push({
        id: frontMatter.id || frontMatter.title,
        metadata: {
          date: date,
          description: frontMatter.description || excerpt,
          permalink: normalizeUrl([
            baseUrl,
            routeBasePath,
            frontMatter.id || linkName,
          ]),
          readingTime: readingStats.text,
          seriesPosition: seriesPosition,
          sort: frontMatter.sort,
          source: aliasedSource,
          tags: tags,
          title: title,
          truncated: truncateMarker?.test(content) || false,
        },
      });
    }),
  );

  return releaseHighlights;
}

export async function generateReleases(
  releaseDir: string,
  {siteConfig, siteDir}: LoadContext,
  options: PluginOptions,
) {
  const {include, routeBasePath, truncateMarker} = options;

  if (!fs.existsSync(releaseDir)) {
    return [];
  }

  const {baseUrl = ''} = siteConfig;
  const releaseFiles = await globby(include, {
    cwd: releaseDir,
  });

  const releases: Release[] = [];

  await Promise.all(
    releaseFiles.map(async (relativeSource: string) => {
      const source = path.join(releaseDir, relativeSource);
      const aliasedSource = aliasedSitePath(source, siteDir);
      const fileString = await fs.readFile(source, 'utf-8');
      const readingStats = readingTime(fileString);
      const {frontMatter, content, excerpt} = parse(fileString);

      if (frontMatter.draft && process.env.NODE_ENV === 'production') {
        return;
      }

      let linkName = relativeSource.replace(/\.mdx?$/, '');
      let seriesPosition = frontMatter.series_position;
      let title = frontMatter.title || linkName;
      let coverLabel = frontMatter.cover_label || title;

      releases.push({
        id: frontMatter.id || frontMatter.title,
        metadata: {
          coverLabel: coverLabel,
          description: frontMatter.description || excerpt,
          permalink: normalizeUrl([
            baseUrl,
            routeBasePath,
            frontMatter.id || linkName,
          ]),
          readingTime: readingStats.text,
          seriesPosition: seriesPosition,
          sort: frontMatter.sort,
          source: aliasedSource,
          title: title,
          truncated: truncateMarker?.test(content) || false,
        },
      });
    }),
  );

  return releases;
}

export function linkify(
  fileContent: string,
  siteDir: string,
  releasePath: string,
  releases: Release[],
) {
  let fencedBlock = false;
  const lines = fileContent.split('\n').map(line => {
    if (line.trim().startsWith('```')) {
      fencedBlock = !fencedBlock;
    }

    if (fencedBlock) return line;

    let modifiedLine = line;
    const mdRegex = /(?:(?:\]\()|(?:\]:\s?))(?!https)([^'")\]\s>]+\.mdx?)/g;
    let mdMatch = mdRegex.exec(modifiedLine);

    while (mdMatch !== null) {
      const mdLink = mdMatch[1];
      const aliasedPostSource = `@site/${path.relative(
        siteDir,
        path.resolve(releasePath, mdLink),
      )}`;
      let releasePermalink = null;

      releases.forEach(release => {
        if (release.metadata.source === aliasedPostSource) {
          releasePermalink = release.metadata.permalink;
        }
      });

      if (releasePermalink) {
        modifiedLine = modifiedLine.replace(mdLink, releasePermalink);
      }

      mdMatch = mdRegex.exec(modifiedLine);
    }

    return modifiedLine;
  });

  return lines.join('\n');
}
