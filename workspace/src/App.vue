<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import projectService from '@/services/projectService';
import LoginView from '@/components/LoginView.vue';
import DashboardView from '@/components/DashboardView.vue';
import FunnelsView from '@/components/FunnelsView.vue';
import AIRecommendationView from '@/components/AIRecommendationView.vue';
import SurveysView from '@/components/SurveysView.vue';
import ProjectList from '@/components/ProjectList.vue';
import ProjectDetail from '@/components/ProjectDetail.vue';
import ProjectForm from '@/components/ProjectForm.vue';
import GA4OAuthCallback from '@/components/GA4OAuthCallback.vue';
import type { Project } from '@/types/project';

// Auth State (Fake)
const isAuthenticated = ref(false);
const userProfile = ref({
  name: 'John Doe',
  email: 'admin@discoveo.ai',
  avatar: 'https://cdn.vuetifyjs.com/images/john.jpg'
});

// Navigation State
type View = 'overview' | 'funnels' | 'qualitative' | 'recommendations' | 'projects' | 'ga4-callback';
const currentView = ref<View>('projects');
const drawer = ref(true);
const rail = ref(false);

const STORAGE_KEY = 'discoveo_active_project_id';

// Global Project State
const projects = ref<Project[]>([]);
const selectedProject = ref<Project | undefined>();
const isLoadingProjects = ref(false);

// Analysis Sub-Page State
type Page = 'list' | 'detail' | 'form';
const currentPage = ref<Page>('list');
const isEditingProject = ref(false);
const showForm = ref(false);
const projectListRef = ref<InstanceType<typeof ProjectList>>();

const fetchGlobalProjects = async () => {
  isLoadingProjects.value = true;
  try {
    projects.value = await projectService.list();

    // Auto-select from localStorage if possible
    const savedId = localStorage.getItem(STORAGE_KEY);
    if (savedId && !selectedProject.value) {
      const found = projects.value.find((p) => p.id === savedId);
      if (found) {
        selectedProject.value = found;
      }
    }
  } catch (err) {
    console.error('Failed to fetch global projects:', err);
  } finally {
    isLoadingProjects.value = false;
  }
};

const handleLogin = async () => {
  isAuthenticated.value = true;
  await fetchGlobalProjects();

  // If no project selected, force list view in Projects
  if (!selectedProject.value) {
    currentView.value = 'projects';
    currentPage.value = 'list';
  } else {
    currentView.value = 'funnels';
  }
};

const handleLogout = () => {
  isAuthenticated.value = false;
  selectedProject.value = undefined;
  localStorage.removeItem(STORAGE_KEY);
  currentView.value = 'projects';
};

// Project Handlers
const handleProjectListItemClick = (project: Project) => {
  selectedProject.value = project;
  currentView.value = 'funnels';
};

const handleProjectSwitch = (project: Project) => {
  selectedProject.value = project;
  localStorage.setItem(STORAGE_KEY, project.id);
  currentView.value = 'funnels';
};

const handleViewProject = (project: Project) => {
  selectedProject.value = project;
  localStorage.setItem(STORAGE_KEY, project.id);
  currentPage.value = 'detail';
  currentView.value = 'projects';
};

const handleProjectsClick = () => {
  currentView.value = 'projects';
  if (selectedProject.value) {
    currentPage.value = 'detail';
  } else {
    currentPage.value = 'list';
  }
};

const handleFormSubmit = async (project: Project) => {
  selectedProject.value = project;
  localStorage.setItem(STORAGE_KEY, project.id);

  isEditingProject.value = false;
  showForm.value = false;
  currentPage.value = 'detail';
  await fetchGlobalProjects();
  projectListRef.value?.fetchProjects();
};

const handleFormCancel = () => {
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

const handleDetailBack = () => {
  currentPage.value = 'list';
};

const handleCreateNew = () => {
  isEditingProject.value = false;
  showForm.value = true;
  currentPage.value = 'form';
};

onMounted(() => {
  if (window.location.pathname === '/ga4-oauth-callback') {
    currentView.value = 'ga4-callback';
  }
});

// Helper for sidebar logic
const isProjectActive = computed(() => !!selectedProject.value);
</script>

<template>
  <v-app v-if="!isAuthenticated && currentView !== 'ga4-callback'">
    <LoginView @login="handleLogin" />
  </v-app>

  <v-app v-else>
    <v-navigation-drawer
      v-model="drawer"
      :rail="rail"
      permanent
      expand-on-hover
      elevation="0"
      class="sidebar-drawer"
      width="280"
      @click="rail = false"
    >
      <div class="pa-6 pb-2 d-flex align-center">
        <v-icon icon="mdi-rocket-launch" color="primary" size="32" class="mr-3" />
        <span v-if="!rail" class="text-h5 font-weight-bold tracking-tight">Discoveo</span>
      </div>



      <v-list nav class="px-4">
        <v-list-item
          prepend-icon="mdi-view-dashboard-outline"
          title="Overview"
          value="overview"
          :active="currentView === 'overview'"
          :disabled="!isProjectActive"
          @click="currentView = 'overview'"
          rounded="lg"
          class="mb-1"
        >
          <template v-slot:append v-if="!isProjectActive">
            <v-icon icon="mdi-lock-outline" size="14" color="grey-lighten-1" />
          </template>
        </v-list-item>

        <v-list-item
          prepend-icon="mdi-filter-variant"
          title="Funnels"
          value="funnels"
          :active="currentView === 'funnels'"
          :disabled="!isProjectActive"
          @click="currentView = 'funnels'"
          rounded="lg"
          class="mb-1"
        >
          <template v-slot:append v-if="!isProjectActive">
            <v-icon icon="mdi-lock-outline" size="14" color="grey-lighten-1" />
          </template>
        </v-list-item>

        <v-list-item
          prepend-icon="mdi-comment-text-outline"
          title="Qualitative Data"
          value="qualitative"
          :active="currentView === 'qualitative'"
          @click="currentView = 'qualitative'"
          rounded="lg"
          class="mb-1"
        />

        <v-list-item
          prepend-icon="mdi-auto-fix"
          title="Recommendations"
          value="recommendations"
          :active="currentView === 'recommendations'"
          :disabled="!isProjectActive"
          @click="currentView = 'recommendations'"
          rounded="lg"
          class="mb-1"
        >
          <template v-slot:append v-if="!isProjectActive">
            <v-icon icon="mdi-lock-outline" size="14" color="grey-lighten-1" />
          </template>
        </v-list-item>

        <v-list-subheader class="text-uppercase text-caption font-weight-bold mt-4">Settings</v-list-subheader>

        <v-list-item
          prepend-icon="mdi-folder-cog-outline"
          title="Projects"
          value="projects"
          :active="currentView === 'projects'"
          @click="handleProjectsClick"
          rounded="lg"
          class="mb-1"
        />
      </v-list>

      <template v-slot:append>
        <div class="pa-4 pt-0" v-if="!rail">
          <v-card variant="tonal" color="primary" class="rounded-xl pa-4 mb-4">
            <div class="text-caption font-weight-bold mb-1">PRO PLAN</div>
            <div class="text-body-2 mb-3">Get unlimited exports and AI insights.</div>
            <v-btn block size="small" variant="flat" color="primary">Upgrade now</v-btn>
          </v-card>

          <!-- Project Switcher moved to bottom -->
          <v-menu width="248">
            <template v-slot:activator="{ props }">
              <v-btn
                v-bind="props"
                block
                variant="outlined"
                color="grey-lighten-1"
                class="project-switcher-btn justify-start px-3"
                height="50"
              >
                <template v-slot:prepend>
                  <v-avatar size="24" color="primary-lighten-4" class="mr-2">
                    <v-icon icon="mdi-folder-outline" size="14" color="primary" />
                  </v-avatar>
                </template>
                <div class="text-left flex-grow-1 overflow-hidden">
                  <div class="text-caption text-grey-darken-1 lh-1 mb-n1">Active Project</div>
                  <div class="text-body-2 font-weight-bold text-truncate text-grey-darken-4">
                    {{ selectedProject?.name || 'Select Project' }}
                  </div>
                </div>
                <template v-slot:append>
                  <v-icon icon="mdi-chevron-down" size="18" />
                </template>
              </v-btn>
            </template>

            <v-list class="pa-2">
              <v-list-subheader class="text-uppercase text-caption font-weight-bold">Your Projects</v-list-subheader>
              <v-list-item
                v-for="project in projects"
                :key="project.id"
                :title="project.name"
                @click="handleProjectSwitch(project)"
                :active="selectedProject?.id === project.id"
                rounded="lg"
              >
                <template v-slot:prepend>
                  <v-icon icon="mdi-folder-outline" size="20" />
                </template>
              </v-list-item>
              <v-divider class="my-2" />
              <v-list-item
                prepend-icon="mdi-plus"
                title="New Project"
                @click="currentView = 'projects'; currentPage = 'form'; isEditingProject = false; showForm = true"
                rounded="lg"
                color="primary"
              />
            </v-list>
          </v-menu>
        </div>
      </template>
    </v-navigation-drawer>

    <v-app-bar elevation="0" class="app-bar px-4" height="70">
      <v-app-bar-nav-icon @click="rail = !rail" />
      
      <v-spacer />

      <v-text-field
        placeholder="Quick search..."
        prepend-inner-icon="mdi-magnify"
        variant="solo"
        density="compact"
        hide-details
        flat
        class="search-field mr-4 d-none d-sm-flex"
      />

      <v-btn icon="mdi-bell-outline" class="mr-2" />
      
      <v-menu min-width="200px" rounded>
        <template v-slot:activator="{ props }">
          <v-btn icon v-bind="props" class="ml-2">
            <v-avatar color="primary" size="36">
              <v-img :src="userProfile.avatar" alt="John" />
            </v-avatar>
          </v-btn>
        </template>
        <v-card>
          <v-list>
            <v-list-item
              :prepend-avatar="userProfile.avatar"
              :title="userProfile.name"
              :subtitle="userProfile.email"
            >
              <template v-slot:append>
                <v-btn icon="mdi-cog" variant="text" size="small" />
              </template>
            </v-list-item>
          </v-list>
          <v-divider />
          <v-list nav>
            <v-list-item prepend-icon="mdi-account-outline" title="Profile" value="profile" />
            <v-list-item prepend-icon="mdi-logout" title="Logout" value="logout" color="error" @click="handleLogout" />
          </v-list>
        </v-card>
      </v-menu>
    </v-app-bar>

    <!-- Main Content Area -->
    <v-main class="main-content bg-grey-lighten-4">
      <v-container fluid class="pa-8">
        <v-fade-transition mode="out-in">
          <!-- GA4 Callback View -->
          <div v-if="currentView === 'ga4-callback'" key="callback">
            <GA4OAuthCallback />
          </div>

          <!-- Overview -->
          <div v-else-if="currentView === 'overview' && selectedProject" key="overview">
            <DashboardView :project="selectedProject" />
          </div>

          <!-- Funnels -->
          <div v-else-if="currentView === 'funnels' && selectedProject" key="funnels">
            <FunnelsView :project="selectedProject" />
          </div>

          <!-- Qualitative Data -->
          <div v-else-if="currentView === 'qualitative'" key="qualitative">
            <SurveysView />
          </div>

          <!-- Recommendations -->
          <div v-else-if="currentView === 'recommendations' && selectedProject" key="recommendations">
            <AIRecommendationView :project="selectedProject" />
          </div>

          <!-- Projects (Settings) -->
          <div v-else-if="currentView === 'projects'" key="projects">


            <v-card class="pa-0 overflow-visible" elevation="0" color="transparent">
              <v-fade-transition mode="out-in">
                <!-- List View -->
                <div v-if="currentPage === 'list'" key="list">
                  <ProjectList
                    ref="projectListRef"
                    @view="handleViewProject"
                    @edit="handleEditProject"
                    @create="handleCreateNew"
                    @refresh="fetchGlobalProjects"
                  />
                </div>

                <!-- Detail View -->
                <div v-else-if="currentPage === 'detail' && selectedProject" key="detail">
                  <ProjectDetail
                    :project="selectedProject"
                    @back="handleDetailBack"
                    @edit="isEditingProject = true; showForm = true; currentPage = 'form'"
                    @delete="currentPage = 'list'; fetchGlobalProjects()"
                  />
                </div>

                <!-- Form View -->
                <div v-else-if="currentPage === 'form' && showForm" key="form">
                  <ProjectForm
                    :project="selectedProject"
                    :is-editing="isEditingProject"
                    @submit="handleFormSubmit"
                    @cancel="handleFormCancel"
                  />
                </div>
              </v-fade-transition>
            </v-card>
          </div>
        </v-fade-transition>
      </v-container>
    </v-main>
  </v-app>
</template>

<style>
/* Global Styles for Iqonic Aesthetic */
body {
  font-family: 'Inter', sans-serif !important;
}

.sidebar-drawer {
  border-right: 1px solid #e2e8f0 !important;
}

.lh-1 { line-height: 1; }

.project-switcher-btn {
  text-transform: none !important;
  border-color: #e2e8f0 !important;
  border-radius: 12px !important;
}

.project-switcher-btn:hover {
  background: #f8fafc !important;
}

.app-bar {
  border-bottom: 1px solid #e2e8f0 !important;
  background: white !important;
}

.search-field {
  max-width: 400px;
}

.search-field .v-field {
  background: #f1f5f9 !important;
  border-radius: 12px !important;
}

.tracking-tight {
  letter-spacing: -0.025em;
}

.main-content {
  min-height: 100vh;
}

/* Custom Vuetify Overrides for Premium Feel */
.v-list-item--active {
  background: rgba(99, 102, 241, 0.1) !important;
  color: #6366f1 !important;
}

.v-list-item--active .v-icon {
  color: #6366f1 !important;
}

.v-btn {
  font-weight: 600 !important;
  letter-spacing: 0px !important;
}

.v-list-subheader {
  font-size: 11px !important;
  color: #94a3b8 !important;
  letter-spacing: 0.05em !important;
}
</style>
