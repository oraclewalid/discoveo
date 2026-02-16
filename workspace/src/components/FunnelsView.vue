<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import type { Project } from '@/types/project'
import type { GA4ConnectorStatus } from '@/types/ga4'
import ga4Service from '@/services/ga4Service'
import FunnelChart from './FunnelChart.vue'
import ScrollDepthChart from './ScrollDepthChart.vue'
import PagePathsTable from './PagePathsTable.vue'
import config from '@/config'

const props = defineProps<{
  project: Project
}>()

const ga4Status = ref<GA4ConnectorStatus | null>(null)
const isLoadingStatus = ref(false)

// Global Filters
const startDate = ref<string>('')
const endDate = ref<string>('')
const selectedDimension = ref('All')
const dimensionOptions = ['device_category', 'country', 'browser', 'operating_system']
const dimensions = computed(() => ['All', ...dimensionOptions])

const isGA4Connected = computed(() => ga4Status.value !== null)

const ga4AuthUrl = computed(() => {
  return `${config.api.baseUrl}projects/${props.project.id}/connectors/ga4/auth/redirect`
})

const fetchGA4Status = async () => {
  isLoadingStatus.value = true
  try {
    ga4Status.value = await ga4Service.getStatus(props.project.id)
  } catch (error) {
    console.error('Failed to fetch GA4 status:', error)
  } finally {
    isLoadingStatus.value = false
  }
}

const handleConnectGA4 = () => {
  window.open(ga4AuthUrl.value, '_blank')
}

onMounted(() => {
  // Initialize dates
  const end = new Date()
  const start = new Date(end.getTime() - 30 * 24 * 60 * 60 * 1000)
  endDate.value = end.toISOString().split('T')[0]!
  startDate.value = start.toISOString().split('T')[0]!
  
  fetchGA4Status()
})

watch(() => props.project.id, () => fetchGA4Status())
</script>

<template>
  <div class="funnels-view">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <h1 class="text-h4 font-weight-bold mb-1">Analytics Dashboard</h1>
        <p class="text-subtitle-1 text-grey">Behavioral insights and conversion performance for {{ project.name }}</p>
      </div>
    </div>

    <!-- Loading Status -->
    <div v-if="isLoadingStatus" class="text-center py-12">
      <v-progress-circular indeterminate color="primary" size="48" />
      <p class="text-grey mt-4">Syncing analytics configuration...</p>
    </div>

    <!-- GA4 Connected: Show Global Filters and Charts -->
    <div v-else-if="isGA4Connected && ga4Status">
      <!-- Global Filters Card -->
      <v-card class="mb-8 filter-card" elevation="0">
        <v-card-text>
          <v-row align="center" dense>
            <v-col cols="12" sm="3">
              <v-text-field
                v-model="startDate"
                label="Start Date"
                type="date"
                variant="solo"
                flat
                density="compact"
                hide-details
                class="custom-field"
              />
            </v-col>

            <v-col cols="12" sm="3">
              <v-text-field
                v-model="endDate"
                label="End Date"
                type="date"
                variant="solo"
                flat
                density="compact"
                hide-details
                class="custom-field"
              />
            </v-col>

            <v-col cols="12" sm="4">
              <v-select
                v-model="selectedDimension"
                label="Dimension Filter"
                :items="dimensions"
                variant="solo"
                flat
                density="compact"
                hide-details
                class="custom-field"
              />
            </v-col>

            <v-col cols="12" sm="2" class="text-right">
              <div class="text-caption text-grey-lighten-1 mb-1 font-weight-bold">REFRESH ALL</div>
              <v-btn
                color="primary"
                variant="tonal"
                block
                prepend-icon="mdi-sync"
                size="small"
                class="rounded-lg"
              >
                Sync Data
              </v-btn>
            </v-col>
          </v-row>
        </v-card-text>
      </v-card>

      <!-- Integrated Charts -->
      <div class="mb-10">
        <FunnelChart 
          :project-id="project.id" 
          :connector-id="ga4Status.connector_id"
          :external-start-date="startDate"
          :external-end-date="endDate"
          :external-dimension="selectedDimension"
        />
      </div>
      <div class="mb-10">
        <ScrollDepthChart 
          :project-id="project.id" 
          :connector-id="ga4Status.connector_id"
          :external-start-date="startDate"
          :external-end-date="endDate"
          :external-dimension="selectedDimension"
        />
      </div>
      <div>
        <PagePathsTable
          :project-id="project.id"
          :connector-id="ga4Status.connector_id"
          :external-start-date="startDate"
          :external-end-date="endDate"
          :external-dimension="selectedDimension"
        />
      </div>
    </div>

    <!-- GA4 Not Connected: Prompt -->
    <v-card v-else class="empty-state-card pa-6 d-flex align-center" elevation="0">
      <v-avatar color="primary-lighten-5" size="48" class="mr-4">
        <v-icon icon="mdi-google-analytics" color="primary" size="24" />
      </v-avatar>
      <div class="flex-grow-1">
        <div class="text-subtitle-1 font-weight-bold">No data source connected</div>
        <p class="text-body-2 text-grey mb-0">Link your GA4 property in Project Settings to view analytics.</p>
      </div>
      <v-btn
        color="primary"
        variant="tonal"
        prepend-icon="mdi-link-variant"
        class="rounded-lg ml-4"
        @click="handleConnectGA4"
      >
        Connect GA4
      </v-btn>
    </v-card>
  </div>
</template>

<style scoped>
.funnels-view {
  width: 100%;
}

.filter-card {
  background: white !important;
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.03) !important;
}

.custom-field :deep(.v-field) {
  background: #f8fafc !important;
  border-radius: 12px !important;
  border: 1px solid #f1f5f9 !important;
}

.empty-state-card {
  border-radius: 24px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}
</style>
