<script setup lang="ts">
import { ref, onMounted } from 'vue';
import ProjectForm from '@/components/ProjectForm.vue';
import ProjectList from '@/components/ProjectList.vue';
import ProjectDetail from '@/components/ProjectDetail.vue';
import GA4OAuthCallback from '@/components/GA4OAuthCallback.vue';
import type { Project } from '@/types/project';

type Page = 'list' | 'detail' | 'form' | 'ga4-callback';

const projectListRef = ref<InstanceType<typeof ProjectList>>();
const currentPage = ref<Page>('list');
const selectedProject = ref<Project | undefined>();
const isEditingProject = ref(false);
const showForm = ref(false);

const handleFormSubmit = (project: Project) => {
  selectedProject.value = undefined;
  isEditingProject.value = false;
  showForm.value = false;
  currentPage.value = 'list';
  projectListRef.value?.fetchProjects();
};

const handleFormCancel = () => {
  selectedProject.value = undefined;
  isEditingProject.value = false;
  showForm.value = false;
  currentPage.value = 'list';
};

const handleEditProject = (project: Project) => {
  selectedProject.value = project;
  isEditingProject.value = true;
  showForm.value = true;
  currentPage.value = 'form';
};

const handleViewProject = (project: Project) => {
  selectedProject.value = project;
  currentPage.value = 'detail';
};

const handleDetailBack = () => {
  selectedProject.value = undefined;
  currentPage.value = 'list';
};

const handleDetailEdit = () => {
  isEditingProject.value = true;
  showForm.value = true;
  currentPage.value = 'form';
};

const handleDetailDelete = () => {
  selectedProject.value = undefined;
  currentPage.value = 'list';
  projectListRef.value?.fetchProjects();
};

const handleCreateNew = () => {
  selectedProject.value = undefined;
  isEditingProject.value = false;
  showForm.value = true;
  currentPage.value = 'form';
};

const handleRefresh = () => {
  projectListRef.value?.fetchProjects();
};

onMounted(() => {
  // Check if user is landing on GA4 callback page
  if (window.location.pathname === '/ga4-oauth-callback') {
    currentPage.value = 'ga4-callback';
  }
});
</script>

<template>
  <div class="app-container">
    <header v-if="currentPage !== 'ga4-callback'" class="app-header">
      <h1>Discoveo - Project Manager</h1>
      <button
        v-if="currentPage !== 'form'"
        @click="handleCreateNew"
        class="btn btn-create"
      >
        + New Project
      </button>
    </header>

    <main class="app-main">
      <!-- GA4 Callback View -->
      <section v-if="currentPage === 'ga4-callback'" class="callback-section">
        <GA4OAuthCallback />
      </section>

      <!-- List View -->
      <section v-else-if="currentPage === 'list'" class="list-section">
        <ProjectList
          ref="projectListRef"
          @view="handleViewProject"
          @edit="handleEditProject"
          @refresh="handleRefresh"
        />
      </section>

      <!-- Detail View -->
      <section v-else-if="currentPage === 'detail' && selectedProject" class="detail-section">
        <ProjectDetail
          :project="selectedProject"
          @back="handleDetailBack"
          @edit="handleDetailEdit"
          @delete="handleDetailDelete"
        />
      </section>

      <!-- Form View -->
      <section v-if="currentPage === 'form' && showForm" class="form-section">
        <ProjectForm
          :project="selectedProject"
          :is-editing="isEditingProject"
          @submit="handleFormSubmit"
          @cancel="handleFormCancel"
        />
      </section>
    </main>
  </div>
</template>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.app-container {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  display: flex;
  flex-direction: column;
}

.app-header {
  background: white;
  border-bottom: 1px solid #e0e0e0;
  padding: 20px 40px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.app-header h1 {
  color: #333;
  font-size: 28px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-create {
  background-color: #4a90e2;
  color: white;
}

.btn-create:hover {
  background-color: #357abd;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(74, 144, 226, 0.3);
}

.app-main {
  flex: 1;
  padding: 40px;
  overflow-y: auto;
}

.list-section,
.detail-section,
.form-section {
  max-width: 1400px;
  margin: 0 auto;
}

.form-section {
  max-width: 600px;
}

/* Responsive Design */
@media (max-width: 968px) {
  .app-header {
    padding: 16px 24px;
    flex-direction: column;
    gap: 16px;
    text-align: center;
  }

  .app-main {
    padding: 24px;
  }
}

@media (max-width: 640px) {
  .app-header h1 {
    font-size: 22px;
  }

  .app-main {
    padding: 16px;
  }

  .btn-create {
    width: 100%;
  }
}
</style>
