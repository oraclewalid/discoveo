/**
 * Project API Service
 * Handles all project-related API calls
 */

import config from '@/config';
import type { Project, CreateProjectDTO, UpdateProjectDTO } from '@/types/project';

const API_ENDPOINT = `${config.api.baseUrl}projects`;

class ProjectService {
  /**
   * Create a new project
   */
  async create(project: CreateProjectDTO): Promise<Project> {
    try {
      const response = await fetch(API_ENDPOINT, {
        method: 'POST',
        headers: config.api.headers || {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(project),
      });

      if (!response.ok) {
        throw new Error(`Failed to create project: ${response.statusText}`);
      }

      const data = await response.json();
      return data as Project;
    } catch (error) {
      console.error('Create project error:', error);
      throw error;
    }
  }

  /**
   * Get all projects
   */
  async list(): Promise<Project[]> {
    try {
      const response = await fetch(API_ENDPOINT, {
        method: 'GET',
        headers: config.api.headers || {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch projects: ${response.statusText}`);
      }

      const data = await response.json();
      return Array.isArray(data) ? data : [];
    } catch (error) {
      console.error('List projects error:', error);
      throw error;
    }
  }

  /**
   * Get a single project by ID
   */
  async getById(id: string): Promise<Project> {
    try {
      const response = await fetch(`${API_ENDPOINT}/${id}`, {
        method: 'GET',
        headers: config.api.headers || {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch project: ${response.statusText}`);
      }

      const data = await response.json();
      return data as Project;
    } catch (error) {
      console.error('Get project error:', error);
      throw error;
    }
  }

  /**
   * Update a project
   */
  async update(project: UpdateProjectDTO): Promise<Project> {
    try {
      const { id, ...updateData } = project;
      const response = await fetch(`${API_ENDPOINT}/${id}`, {
        method: 'PUT',
        headers: config.api.headers || {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(updateData),
      });

      if (!response.ok) {
        throw new Error(`Failed to update project: ${response.statusText}`);
      }

      const data = await response.json();
      return data as Project;
    } catch (error) {
      console.error('Update project error:', error);
      throw error;
    }
  }

  /**
   * Delete a project
   */
  async delete(id: string): Promise<void> {
    try {
      const response = await fetch(`${API_ENDPOINT}/${id}`, {
        method: 'DELETE',
        headers: config.api.headers || {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to delete project: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Delete project error:', error);
      throw error;
    }
  }
}

export default new ProjectService();
