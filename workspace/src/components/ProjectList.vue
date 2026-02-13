<template>
  <div class="project-list">
    <h2>Projects</h2>

    <div v-if="isLoading" class="loading">
      Loading projects...
    </div>

    <div v-else-if="projects.length === 0" class="empty-state">
      <p>No projects found. Create one to get started!</p>
    </div>

    <div v-else class="projects-grid">
      <div
        v-for="project in projects"
        :key="project.id"
        class="project-card"
        @click="handleView(project)"
      >
        <div class="card-header">
          <h3>{{ project.name }}</h3>
        </div>

        <p v-if="project.description" class="description">
          {{ project.description }}
        </p>

        <div class="card-actions" @click.stop>
          <button @click="handleEdit(project)" class="btn btn-sm btn-edit">
            Edit
          </button>
          <button @click="handleDelete(project.id)" class="btn btn-sm btn-delete">
            Delete
          </button>
        </div>
      </div>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import projectService from '@/services/projectService';
import type { Project } from '@/types/project';

interface Emits {
  (e: 'view', project: Project): void;
  (e: 'edit', project: Project): void;
  (e: 'refresh'): void;
}

const emit = defineEmits<Emits>();

const projects = ref<Project[]>([]);
const isLoading = ref(false);
const error = ref('');

const fetchProjects = async () => {
  isLoading.value = true;
  error.value = '';

  try {
    projects.value = await projectService.list();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load projects';
    console.error('Fetch projects error:', err);
  } finally {
    isLoading.value = false;
  }
};

const handleEdit = (project: Project) => {
  emit('edit', project);
};

const handleView = (project: Project) => {
  emit('view', project);
};

const handleDelete = async (id: string) => {
  if (!confirm('Are you sure you want to delete this project?')) {
    return;
  }

  try {
    await projectService.delete(id);
    await fetchProjects();
    emit('refresh');
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to delete project';
  }
};

onMounted(() => {
  fetchProjects();
});

defineExpose({
  fetchProjects,
});
</script>

<style scoped>
.project-list {
  width: 100%;
}

.project-list h2 {
  margin-bottom: 20px;
  color: #333;
  font-size: 24px;
}

.loading,
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: #666;
  font-size: 16px;
}

.projects-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.project-card {
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  cursor: pointer;
}

.project-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: 12px;
}

.card-header h3 {
  margin: 0;
  color: #333;
  font-size: 18px;
  flex: 1;
}

.description {
  color: #666;
  font-size: 14px;
  margin: 12px 0;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.card-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  flex: 1;
}

.btn-sm {
  padding: 6px 12px;
}

.btn-edit {
  background-color: #4a90e2;
  color: white;
}

.btn-edit:hover {
  background-color: #357abd;
}

.btn-delete {
  background-color: #e74c3c;
  color: white;
}

.btn-delete:hover {
  background-color: #c0392b;
}

.error-message {
  padding: 12px;
  background-color: #ffe0e0;
  color: #e74c3c;
  border-radius: 4px;
  margin-top: 16px;
  font-size: 14px;
}
</style>
