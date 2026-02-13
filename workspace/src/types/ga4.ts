/**
 * GA4 Connector Types
 */

export interface GA4ConnectorStatus {
  is_expired: boolean;
  connector_id: string;
  expires_at: string;
  propertyId?: string;
  propertyName?: string;
  lastSync?: string;
}

export interface GA4Property {
  name: string;
  display_name: string;
  property_type: string;
}

export interface GA4Properties {
  properties: GA4Property[];
}
