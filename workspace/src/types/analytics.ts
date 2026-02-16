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

export interface SurveyStats {
  total_responses: number;
  average_rating: number;
  first_response_date: string;
  last_response_date: string;
  responses_with_comments: number;
}
