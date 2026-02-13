/**
 * GA4 Connector Service
 * Handles all GA4 connector-related API calls
 */

import config from '@/config';
import type { GA4ConnectorStatus, GA4Properties, GA4Property } from '@/types/ga4';

class GA4ConnectorService {
  /**
   * Get GA4 connector status for a project
   */
  async getStatus(projectId: string): Promise<GA4ConnectorStatus | null> {
    try {
      const response = await fetch(
        `${config.api.baseUrl}projects/${projectId}/connectors/ga4/status`,
        {
          method: 'GET',
          headers: config.api.headers || {
            'Content-Type': 'application/json',
          },
        }
      );

      // 404 means not connected
      if (response.status === 404) {
        return null;
      }

      if (!response.ok) {
        throw new Error(`Failed to fetch GA4 status: ${response.statusText}`);
      }

      const data = await response.json();
      return data as GA4ConnectorStatus;
    } catch (error) {
      console.error('Get GA4 status error:', error);
      // If it's a 404, return null (not connected)
      if (error instanceof Error && error.message.includes('404')) {
        return null;
      }
      throw error;
    }
  }

  /**
   * Get GA4 properties for a project
   */
  async getProperties(projectId: string): Promise<GA4Property[]> {
    try {
      const response = await fetch(
        `${config.api.baseUrl}projects/${projectId}/connectors/ga4/properties`,
        {
          method: 'GET',
          headers: config.api.headers || {
            'Content-Type': 'application/json',
          },
        }
      );

      if (!response.ok) {
        throw new Error(`Failed to fetch GA4 properties: ${response.statusText}`);
      }

      const data = await response.json();

      // API returns array directly, not wrapped in an object
      if (Array.isArray(data)) {
        return data as GA4Property[];
      }

      // Fallback: if wrapped in an object, use properties array
      return (data as GA4Properties).properties || [];
    } catch (error) {
      console.error('Get GA4 properties error:', error);
      throw error;
    }
  }

  /**
   * Select a GA4 property for a connector
   */
  async selectProperty(projectId: string, connectorId: string, propertyName: string, propertyId: string): Promise<void> {
    try {
      const response = await fetch(
        `${config.api.baseUrl}projects/${projectId}/connectors/ga4/${connectorId}/property`,
        {
          method: 'PUT',
          headers: config.api.headers || {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ property_id: propertyId, property_name: propertyName }),
        }
      );

      if (!response.ok) {
        throw new Error(`Failed to select GA4 property: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Select GA4 property error:', error);
      throw error;
    }
  }

  /**
   * Pull data from GA4
   */
  async pullData(projectId: string, connectorId: string): Promise<void> {
    try {
      const response = await fetch(
        `${config.api.baseUrl}projects/${projectId}/connectors/ga4/${connectorId}/pull`,
        {
          method: 'POST',
          headers: config.api.headers || {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({}),
        }
      );

      if (!response.ok) {
        throw new Error(`Failed to pull data from GA4: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Pull GA4 data error:', error);
      throw error;
    }
  }

  /**
   * Disconnect GA4 connector
   */
  async disconnect(projectId: string): Promise<void> {
    try {
      const response = await fetch(
        `${config.api.baseUrl}projects/${projectId}/connectors/ga4/disconnect`,
        {
          method: 'GET',
          headers: config.api.headers || {
            'Content-Type': 'application/json',
          },
        }
      );

      if (!response.ok) {
        throw new Error(`Failed to disconnect GA4: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Disconnect GA4 error:', error);
      throw error;
    }
  }
}

export default new GA4ConnectorService();
