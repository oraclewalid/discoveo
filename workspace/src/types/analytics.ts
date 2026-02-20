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

export interface SurveyFeedback {
  id: string;
  date: string;
  country: string;
  url: string;
  device: string;
  browser: string;
  os: string;
  rating: number;
  comment: string;
}

export interface SurveyTheme {
  name: string;
  description: string;
  sentiment: 'positive' | 'negative' | 'neutral' | 'mixed';
  frequency: 'high' | 'medium' | 'low';
  sample_quotes: string[];
}

export interface SentimentBreakdown {
  positive_pct: number;
  negative_pct: number;
  neutral_pct: number;
}

export interface KeyIssue {
  title: string;
  severity: 'critical' | 'major' | 'minor';
  description: string;
  affected_users_pct: number;
}

export interface Recommendation {
  title: string;
  priority: 'high' | 'medium' | 'low';
  description: string;
  expected_impact: string;
}

export interface SurveyAnalysis {
  themes: SurveyTheme[];
  sentiment_breakdown: SentimentBreakdown;
  key_issues: KeyIssue[];
  recommendations: Recommendation[];
}

export interface SurveyReport {
  id: string;
  project_id: string;
  created_at: string;
  analysis: SurveyAnalysis;
  narrative: string;
  model_used: string;
}
