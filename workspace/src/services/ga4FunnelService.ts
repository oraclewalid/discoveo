/**
 * GA4 Funnel API Service
 * Handles funnel-related API calls to GA4 connector
 */

import config from '@/config'
import type { FunnelData } from '@/types/funnel'

class GA4FunnelService {
  /**
   * Get funnel data with filters
   */
  async getFunnelData(
    projectId: string,
    connectorId: string,
    dimension: string,
    startDate: string,
    endDate: string,
  ): Promise<FunnelData> {
    try {
      const url = new URL(
        `${config.api.baseUrl}projects/${projectId}/connectors/ga4/${connectorId}/funnel`,
      )
      url.searchParams.append('dimension', dimension)
      url.searchParams.append('start_date', startDate)
      url.searchParams.append('end_date', endDate)

      console.log('API URL:', url.toString())

      const response = await fetch(url.toString(), {
        method: 'GET',
        headers: config.api.headers || {
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        const errorBody = await response.text()
        console.error('API Error Response:', errorBody)
        throw new Error(`Failed to fetch funnel data: ${response.statusText}`)
      }

      const data = await response.json()
      console.log('Raw API response:', data)

      // Handle both direct array and wrapped response
      const funnelArray = Array.isArray(data) ? data : data.data || data

      if (!Array.isArray(funnelArray)) {
        throw new Error('Invalid response format: expected array')
      }

      return funnelArray as FunnelData
    } catch (error) {
      console.error('Get funnel data error:', error)
      throw error
    }
  }
}

export default new GA4FunnelService()
