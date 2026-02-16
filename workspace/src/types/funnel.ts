/**
 * Funnel Types and Interfaces
 */

export interface FunnelStageData {
  stage_order: number
  dimension: string
  funnel_stage: string
  total_users: number
  total_interactions: number
  prev_stage_users: number | null
  users_dropped: number | null
  dropoff_pct: number | null
  conversion_from_start_pct: number
  stage_conversion_pct: number | null
  ranking: number
}

export interface FunnelData extends Array<FunnelStageData> {}

export interface FunnelQuery {
  projectId: string
  connectorId: string
  dimension: string
  startDate: string
  endDate: string
}
