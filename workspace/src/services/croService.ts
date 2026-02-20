import config from '@/config'

export interface CROReport {
  id: string
  project_id: string
  status: string
  createdAt: string
  analysis?: any
}

class CROService {
  private get baseUrl() {
    return `${config.api.baseUrl}projects/`
  }

  async generateReport(projectId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}${projectId}/cro/report`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(config.api.headers || {})
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to generate CRO report: ${response.statusText}`)
    }
  }

  async listReports(projectId: string): Promise<CROReport[]> {
    const response = await fetch(`${this.baseUrl}${projectId}/cro/reports`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...(config.api.headers || {})
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to fetch CRO reports: ${response.statusText}`)
    }

    return await response.json()
  }

  async getReport(projectId: string, reportId: string): Promise<CROReport> {
    const response = await fetch(`${this.baseUrl}${projectId}/cro/reports/${reportId}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        ...(config.api.headers || {})
      },
    })

    if (!response.ok) {
      throw new Error(`Failed to fetch CRO report: ${response.statusText}`)
    }

    return await response.json()
  }
}

export default new CROService()
