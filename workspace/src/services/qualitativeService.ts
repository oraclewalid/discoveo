import config from '@/config'
import type { SurveyStats } from '@/types/analytics'

class QualitativeService {
  /**
   * Get survey statistics for a project
   */
  async getSurveyStats(projectId: string): Promise<SurveyStats> {
    try {
      const response = await fetch(`${config.api.baseUrl}projects/${projectId}/qualitative/stats`, {
        method: 'GET',
        headers: config.api.headers || {
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        throw new Error(`Failed to fetch survey stats: ${response.statusText}`)
      }

      return await response.json()
    } catch (error) {
      console.error('Get survey stats error:', error)
      throw error
    }
  }
}

export default new QualitativeService()
