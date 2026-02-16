<template>
  <div class="project-detail">
    <!-- Breadcrumb-style Back Navigation -->
    <div class="d-flex align-center mb-6">
      <v-btn
        @click="handleBack"
        variant="text"
        color="grey-darken-1"
        prepend-icon="mdi-chevron-left"
        class="text-none px-0 mr-4"
      >
        Back to projects
      </v-btn>
      <v-divider vertical class="mx-2" style="height: 20px" />
      <span class="text-body-2 font-weight-medium text-grey ml-4">
        Projects / {{ project?.name || 'Loading...' }}
      </span>
    </div>

    <!-- Loading State -->
    <div v-if="!project" class="text-center py-12">
      <v-progress-circular indeterminate color="primary" size="64" />
    </div>

    <!-- Project Details -->
    <div v-else class="detail-container">
      <v-row>
        <!-- Left Column: Main Info -->
        <v-col cols="12" lg="8">
          <v-card class="premium-detail-card mb-6" elevation="0">
            <v-card-text class="pa-8">
              <div class="d-flex justify-space-between align-start mb-6">
                <div>
                  <h1 class="text-h3 font-weight-bold tracking-tight mb-2">
                    {{ project.name }}
                  </h1>
                  <v-chip color="primary" variant="flat" size="small" class="font-weight-bold px-4">
                    ACTIVE PROJECT
                  </v-chip>
                </div>
                <div class="d-flex gap-2">
                  <v-btn icon="mdi-pencil-outline" variant="tonal" color="primary" @click="handleEdit" />
                  <v-btn icon="mdi-delete-outline" variant="tonal" color="error" @click="handleDelete" />
                </div>
              </div>

              <div class="description-section mt-8">
                <div class="text-overline text-primary font-weight-bold mb-2">Project Description</div>
                <p v-if="project.description" class="text-h6 font-weight-regular text-grey-darken-2 lh-relaxed">
                  {{ project.description }}
                </p>
                <p v-else class="text-body-1 text-grey font-italic">No description provided for this project.</p>
              </div>

              <v-divider class="my-8 opacity-10" />

              <div class="d-flex align-center">
                <div class="project-meta-item mr-8">
                  <div class="text-caption text-grey font-weight-bold text-uppercase">Project ID</div>
                  <div class="text-body-1 font-weight-bold text-mono">{{ project.id }}</div>
                </div>
                <div class="project-meta-item">
                  <div class="text-caption text-grey font-weight-bold text-uppercase">Status</div>
                  <div class="d-flex align-center">
                    <div class="status-dot bg-success mr-2"></div>
                    <span class="text-body-1 font-weight-bold">Healthy</span>
                  </div>
                </div>
              </div>
            </v-card-text>
          </v-card>

        </v-col>

        <!-- Right Column: Connectors & Actions -->
        <v-col cols="12" lg="4">
          <v-card class="side-card mb-6" elevation="0">
            <v-card-title class="pa-6 border-bottom">
              <span class="text-h6 font-weight-bold">Data Sources</span>
            </v-card-title>
            
            <v-card-text class="pa-6">
              <!-- Loading GA4 Status -->
              <div v-if="isLoadingGA4" class="py-4">
                <v-progress-linear indeterminate color="primary" rounded height="6" />
                <p class="text-caption text-center text-grey mt-2">Syncing connector status...</p>
              </div>

              <!-- GA4 Connected State -->
              <div v-else-if="isGA4Connected && ga4Status" class="ga4-premium-widget">
                <div class="widget-header d-flex align-center mb-4">
                  <v-avatar color="primary-lighten-5" rounded="lg" size="48" class="mr-4">
                    <v-icon icon="mdi-google-analytics" color="primary" />
                  </v-avatar>
                  <div>
                    <div class="text-subtitle-2 font-weight-bold">GA4 Connector</div>
                    <div class="d-flex align-center">
                      <span class="text-caption" :class="ga4Status.is_expired ? 'text-warning' : 'text-success'">
                         {{ ga4Status.is_expired ? 'Expired' : 'Live Syncing' }}
                      </span>
                    </div>
                  </div>
                </div>

                <v-list density="compact" class="bg-transparent mb-4">
                  <v-list-item class="px-0">
                    <template v-slot:prepend><v-icon icon="mdi-domain" size="16" class="mr-2 opacity-50" /></template>
                    <v-list-item-subtitle class="text-caption">Property</v-list-item-subtitle>
                    <v-list-item-title class="text-body-2 font-weight-bold">{{ ga4Status.propertyName || 'N/A' }}</v-list-item-title>
                  </v-list-item>
                  <v-list-item class="px-0">
                    <template v-slot:prepend><v-icon icon="mdi-sync" size="16" class="mr-2 opacity-50" /></template>
                    <v-list-item-subtitle class="text-caption">Last Sync</v-list-item-subtitle>
                    <v-list-item-title class="text-body-2 font-weight-bold">
                      {{ ga4Status.lastSync ? new Date(ga4Status.lastSync).toLocaleDateString() : 'Never' }}
                    </v-list-item-title>
                  </v-list-item>
                </v-list>

                <v-btn block color="primary" variant="flat" class="mb-2" prepend-icon="mdi-sync" @click="handlePullData">
                  Sync Now
                </v-btn>
                <v-btn block color="error" variant="text" size="small" @click="handleDisconnectGA4">
                  Disconnect Source
                </v-btn>
              </div>

              <!-- GA4 Not Connected State -->
              <v-card v-else variant="outlined" color="primary" class="border-dashed bg-primary-lighten-5 pa-6 text-center" @click="handleConnectGA4">
                <v-icon icon="mdi-google-analytics" size="48" class="mb-4" />
                <div class="text-subtitle-1 font-weight-bold mb-1">Add Google Analytics</div>
                <p class="text-caption mb-4">Integrate your funnel data automatically.</p>
                <v-btn color="primary" variant="flat" block>Connect</v-btn>
              </v-card>
            </v-card-text>
          </v-card>

          <!-- Quick Actions -->
          <v-card class="side-card" elevation="0">
            <v-card-title class="pa-6 border-bottom">
              <span class="text-h6 font-weight-bold">Quick Insights</span>
            </v-card-title>
            <v-card-text class="pa-6">
              <div v-for="action in quickActions" :key="action.title" class="mb-4">
                <v-btn block variant="tonal" :color="action.color" class="justify-start px-4 h-auto py-3">
                  <template v-slot:prepend>
                    <v-avatar :color="action.color + '-lighten-4'" size="32" class="mr-2">
                      <v-icon :icon="action.icon" size="18" :color="action.color" />
                    </v-avatar>
                  </template>
                  <div class="text-left">
                    <div class="text-subtitle-2 font-weight-bold">{{ action.title }}</div>
                    <div class="text-caption opacity-70">{{ action.desc }}</div>
                  </div>
                </v-btn>
              </div>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </div>

    <!-- Dialogs -->
    <v-dialog v-model="deleteDialog" max-width="450">
      <v-card class="rounded-xl pa-4">
        <v-card-title class="text-h5 font-weight-bold">Delete Project?</v-card-title>
        <v-card-text>This will permanently remove the project and all its connection data.</v-card-text>
        <v-card-actions class="mt-4">
          <v-spacer />
          <v-btn variant="text" color="grey" @click="deleteDialog = false">Cancel</v-btn>
          <v-btn variant="flat" color="error" class="px-6" @click="confirmDelete">Yes, Delete</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="disconnectDialog" max-width="450">
      <v-card class="rounded-xl pa-4">
        <v-card-title class="text-h5 font-weight-bold">Disconnect GA4?</v-card-title>
        <v-card-text>You will stop receiving updates, but historical data will remain.</v-card-text>
        <v-card-actions class="mt-4">
          <v-spacer />
          <v-btn variant="text" color="grey" @click="disconnectDialog = false">Keep Connected</v-btn>
          <v-btn variant="flat" color="warning" class="px-6" @click="confirmDisconnect">Disconnect</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-snackbar v-model="pullDataSnackbar" color="success" class="custom-snackbar">
      <v-icon icon="mdi-check-circle" class="mr-2" />
      Syncing funnel data...
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import type { Project } from '@/types/project'
import type { GA4ConnectorStatus } from '@/types/ga4'
import projectService from '@/services/projectService'
import ga4Service from '@/services/ga4Service'
import config from '@/config'

interface Props {
  project: Project
}

interface Emits {
  (e: 'back'): void
  (e: 'edit'): void
  (e: 'delete'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const project = computed(() => props.project)
const ga4Status = ref<GA4ConnectorStatus | null>(null)
const isLoadingGA4 = ref(false)
const ga4Error = ref('')
const deleteDialog = ref(false)
const disconnectDialog = ref(false)
const pullDataSnackbar = ref(false)

const ga4AuthUrl = computed(() => {
  return `${config.api.baseUrl}projects/${project.value.id}/connectors/ga4/auth/redirect`
})

const isGA4Connected = computed(() => ga4Status.value !== null)

const quickActions = [
  { title: 'AI Audit', desc: 'Run automated project quality check.', icon: 'mdi-robot-confused', color: 'primary' },
  { title: 'Export PDF', desc: 'Generate monthly performance report.', icon: 'mdi-file-pdf-box', color: 'secondary' },
  { title: 'Team Access', desc: 'Manage project collaborators.', icon: 'mdi-account-group', color: 'accent' },
]

const handleBack = () => emit('back')
const handleEdit = () => emit('edit')
const handleDelete = () => deleteDialog.value = true

const handleConnectGA4 = () => {
  window.open(ga4AuthUrl.value, '_blank')
}

const confirmDelete = async () => {
  try {
    await projectService.delete(project.value.id)
    deleteDialog.value = false
    emit('delete')
  } catch (error) {
    ga4Error.value = error instanceof Error ? error.message : 'Failed to delete'
  }
}

const handleDisconnectGA4 = () => disconnectDialog.value = true

const confirmDisconnect = async () => {
  try {
    await ga4Service.disconnect(project.value.id)
    ga4Status.value = null
    disconnectDialog.value = false
  } catch (error) {
    ga4Error.value = error instanceof Error ? error.message : 'Failed to disconnect'
  }
}

const handlePullData = async () => {
  if (!ga4Status.value) return
  try {
    await ga4Service.pullData(project.value.id, ga4Status.value.connector_id)
    pullDataSnackbar.value = true
  } catch (error) {
    ga4Error.value = error instanceof Error ? error.message : 'Failed to pull'
  }
}

const fetchGA4Status = async () => {
  isLoadingGA4.value = true
  try {
    const status = await ga4Service.getStatus(project.value.id)
    ga4Status.value = status
  } catch (error) {
    console.error('Failed to fetch GA4 status:', error)
  } finally {
    isLoadingGA4.value = false
  }
}

onMounted(() => fetchGA4Status())
</script>

<style scoped>
.project-detail {
  width: 100%;
}

.premium-detail-card {
  border-radius: 24px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}

.side-card {
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}

.border-bottom {
  border-bottom: 1px solid #f1f5f9;
}

.lh-relaxed {
  line-height: 1.6;
}

.text-mono {
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.ga4-premium-widget {
  background: #f8fafc;
  border-radius: 16px;
  padding: 16px;
  border: 1px solid #f1f5f9;
}

.border-dashed {
  border: 2px dashed #e2e8f0 !important;
  cursor: pointer;
  transition: all 0.2s ease;
}

.border-dashed:hover {
  background: #f1f5f9 !important;
  border-color: #6366f1 !important;
}

.custom-snackbar :deep(.v-snackbar__wrapper) {
  border-radius: 12px !important;
  box-shadow: 0 8px 16px rgba(0,0,0,0.1);
}

.tracking-tight {
  letter-spacing: -0.025em;
}

.gap-2 {
  gap: 8px;
}
</style>
