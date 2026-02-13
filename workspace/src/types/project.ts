/**
 * Project Types and Interfaces
 * Matches Rust struct: Project { id: Uuid, name: String, description: Option<String> }
 */

export interface Project {
  id: string; // Uuid serialized as string
  name: string;
  description?: string | null;
}

export interface CreateProjectDTO {
  name: string;
  description?: string | null;
}

export interface UpdateProjectDTO extends Partial<CreateProjectDTO> {
  id: string;
}
