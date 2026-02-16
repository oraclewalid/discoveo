export interface ScrollDepthData {
  scroll_depth: number;
  users: number;
  events: number;
  users_lost: number | null;
  drop_off_pct: number | null;
  dimension?: string;
}

export interface ScrollDepthResponse extends Array<ScrollDepthData> {}

export interface PagePathData {
  page_path: string;
  total_pageviews: number;
  total_users: number;
  total_engagement_seconds: number;
  avg_time_per_pageview_sec: number;
  avg_time_per_user_sec: number;
  dimension?: string;
}
