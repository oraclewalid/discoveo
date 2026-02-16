<template>
  <div class="project-list">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <h2 class="text-h3 font-weight-bold tracking-tight mb-1">
          Projects
        </h2>
        <p class="text-subtitle-1 text-grey-darken-1">Manage and analyze your active marketing funnels</p>
      </div>
    </div>

    <!-- Loading State -->
    <v-row v-if="isLoading">
      <v-col v-for="n in 3" :key="n" cols="12" sm="6" md="4">
        <v-skeleton-loader type="card" class="rounded-xl" />
      </v-col>
    </v-row>

    <!-- Project Grid -->
    <v-row v-else>
      <!-- Create New Card -->
      <v-col cols="12" sm="6" md="4">
        <v-card
          class="create-card d-flex flex-column align-center justify-center text-center pa-6 h-100"
          elevation="0"
          @click="$emit('create')"
        >
          <v-avatar color="primary-lighten-5" size="64" class="mb-4">
            <v-icon icon="mdi-plus" color="primary" size="32" />
          </v-avatar>
          <div class="text-h6 font-weight-bold mb-1">New Project</div>
          <p class="text-caption text-grey">Launch a new analysis context</p>
        </v-card>
      </v-col>

      <!-- Empty State (if no projects and not loading) -->
      <template v-if="projects.length === 0">
        <!-- Already covered by Create New card -->
      </template>

      <!-- Projects -->
      <v-col
        v-for="project in projects"
        :key="project.id"
        cols="12"
        sm="6"
        md="4"
      >
        <v-card
          class="premium-project-card h-100 d-flex flex-column"
          elevation="0"
          @click="handleView(project)"
        >
          <v-card-text class="pa-6 flex-grow-1">
            <div class="d-flex align-center justify-space-between mb-4">
              <v-avatar color="primary-lighten-5" rounded="lg" size="48">
                <v-icon icon="mdi-rocket-launch-outline" color="primary" size="24" />
              </v-avatar>
              <div class="d-flex gap-1">
                <v-btn icon="mdi-pencil-outline" variant="text" size="small" color="grey" @click.stop="handleEdit(project)" />
                <v-btn icon="mdi-delete-outline" variant="text" size="small" color="error" @click.stop="handleDelete(project.id)" />
              </div>
            </div>

            <h3 class="text-h5 font-weight-bold text-truncate mb-2">{{ project.name }}</h3>
            <p class="description text-body-2 text-grey-darken-1 mb-4">
              {{ project.description || 'No description provided for this project.' }}
            </p>

            <v-divider class="my-4 opacity-10" />

            <div class="d-flex align-center justify-space-between">
              <div class="d-flex align-center">
                <v-icon icon="mdi-pulse" size="14" color="success" class="mr-1" />
                <span class="text-caption font-weight-bold text-success">Healthy</span>
              </div>
              <span class="text-caption text-grey-darken-1 font-weight-bold">ID: {{ project.id.substring(0, 8) }}</span>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="deleteDialog" max-width="450">
      <v-card class="rounded-xl pa-4">
        <v-card-title class="text-h5 font-weight-bold">Delete Project?</v-card-title>
        <v-card-text>This action cannot be undone. All associated data will be lost.</v-card-text>
        <v-card-actions class="mt-4">
          <v-spacer />
          <v-btn variant="text" color="grey" @click="deleteDialog = false">Cancel</v-btn>
          <v-btn variant="flat" color="error" class="px-6 rounded-lg" @click="confirmDelete">Yes, Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import projectService from '@/services/projectService';
import type { Project } from '@/types/project';

interface Emits {
  (e: 'view', project: Project): void;
  (e: 'edit', project: Project): void;
  (e: 'create'): void;
  (e: 'refresh'): void;
}

const emit = defineEmits<Emits>();

const projects = ref<Project[]>([]);
const isLoading = ref(false);
const error = ref('');
const showError = ref(false);
const deleteDialog = ref(false);
const projectToDelete = ref<string | null>(null);

const fetchProjects = async () => {
  isLoading.value = true;
  try {
    projects.value = await projectService.list();
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load projects';
    showError.value = true;
  } finally {
    isLoading.value = false;
  }
};

const handleEdit = (project: Project) => emit('edit', project);
const handleView = (project: Project) => emit('view', project);
const handleDelete = (id: string) => {
  projectToDelete.value = id;
  deleteDialog.value = true;
};

const confirmDelete = async () => {
  if (!projectToDelete.value) return;
  try {
    await projectService.delete(projectToDelete.value);
    deleteDialog.value = false;
    await fetchProjects();
    emit('refresh');
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to delete';
    showError.value = true;
  }
};

onMounted(() => fetchProjects());
defineExpose({ fetchProjects });
</script>

<style scoped>
.project-list {
  width: 100%;
}

.premium-project-card {
  border-radius: 24px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
}

.premium-project-card:hover {
  transform: translateY(-5px);
  border-color: #6366f1 !important;
  box-shadow: 0 12px 24px rgba(99, 102, 241, 0.1) !important;
}

.create-card {
  border-radius: 24px !important;
  border: 2px dashed #e2e8f0 !important;
  background: #f8fafc !important;
  cursor: pointer;
  transition: all 0.25s ease;
}

.create-card:hover {
  border-color: #6366f1 !important;
  background: #f1f5f9 !important;
}

.description {
  line-height: 1.6;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 44px;
}

.tracking-tight {
  letter-spacing: -0.025em;
}

.gap-1 {
  gap: 4px;
}
</style>
