/**
 * Application Configuration
 * Centralizes all configuration with strong typing
 */

/**
 * API Configuration
 */
interface ApiConfig {
  baseUrl: string;
  timeout?: number;
  retries?: number;
  headers?: Record<string, string>;
}

/**
 * Authentication Configuration
 */
interface AuthConfig {
  tokenKey: string;
  refreshTokenKey: string;
  tokenExpiry?: number;
}

/**
 * Application Metadata Configuration
 */
interface AppMetaConfig {
  name: string;
  version: string;
  environment: 'development' | 'staging' | 'production';
}

/**
 * Logger Configuration
 */
interface LoggerConfig {
  level: 'debug' | 'info' | 'warn' | 'error';
  enabled: boolean;
}

/**
 * Main Application Configuration
 */
interface IAppConfig {
  api: ApiConfig;
  auth?: AuthConfig;
  app?: AppMetaConfig;
  logger?: LoggerConfig;
}

/**
 * Application Configuration Instance
 */
const config: IAppConfig = {
  api: {
    baseUrl: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api',
    timeout: 30000,
    retries: 3,
    headers: {
      'Content-Type': 'application/json',
    },
  },
  auth: {
    tokenKey: 'auth_token',
    refreshTokenKey: 'refresh_token',
    tokenExpiry: 3600,
  },
  app: {
    name: 'Discoveo',
    version: '1.0.0',
    environment: (import.meta.env.MODE as 'development' | 'staging' | 'production') || 'development',
  },
  logger: {
    level: 'info',
    enabled: true,
  },
};

/**
 * Get immutable config object
 */
export const getConfig = (): Readonly<IAppConfig> => Object.freeze(JSON.parse(JSON.stringify(config)));

/**
 * Export types and config
 */
export type { IAppConfig, ApiConfig, AuthConfig, AppMetaConfig, LoggerConfig };
export default config;
