//
// Generic
//

export interface PluginOptions {
  path: string;
  routeBasePath: string;
  include: string[];
  releaseComponent: string;
  releaseDownloadComponent: string;
  releaseHighlightComponent: string;
  releaseHighlightsListComponent: string;
  releaseListComponent: string;
  remarkPlugins: string[];
  rehypePlugins: string[];
  truncateMarker: RegExp;
}

export interface Paginator {
  title: string;
  permalink: string;
}

export interface ReleaseContent {
  releases: Release[];
  releaseHighlights: ReleaseHighlight[];
}

export interface Tag {
  label: string;
  permalink: string;
}

//
// Highlight
//

export interface ReleaseHighlight {
  id: string;
  metadata: ReleaseHighlightMetaData;
}

export interface ReleaseHighlightMetaData {
  date: string;
  description: string;
  nextItem?: Paginator;
  permalink: string;
  prevItem?: Paginator;
  readingTime: string;
  seriesPosition: number;
  sort: number;
  source: string;
  tags: (Tag | string)[];
  title: string;
  truncated: boolean;
}

//
// Release
//

export interface Release {
  id: string;
  metadata: ReleaseMetaData;
}

export interface ReleaseMetaData {
  coverLabel: string;
  description: string;
  nextItem?: Paginator;
  permalink: string;
  prevItem?: Paginator;
  readingTime: string;
  seriesPosition: number;
  sort: number;
  source: string;
  title: string;
  truncated: boolean;
}
