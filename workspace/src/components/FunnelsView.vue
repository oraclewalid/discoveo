<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import type { Project } from '@/types/project'
import type { GA4ConnectorStatus } from '@/types/ga4'
import ga4Service from '@/services/ga4Service'
import FunnelChart from './FunnelChart.vue'
import config from '@/config'

const props = defineProps<{
  project: Project
}>()

const ga4Status = ref<GA4ConnectorStatus | null>(null)
const isLoading = ref(false)

const isGA4Connected = computed(() => ga4Status.value !== null)

const ga4AuthUrl = computed(() => {
  return `${config.api.baseUrl}projects/${props.project.id}/connectors/ga4/auth/redirect`
})

const fetchGA4Status = async () => {
  isLoading.value = true
  try {
    ga4Status.value = await ga4Service.getStatus(props.project.id)
  } catch (error) {
    console.error('Failed to fetch GA4 status:', error)
  } finally {
    isLoading.value = false
  }
}

const handleConnectGA4 = () => {
  window.open(ga4AuthUrl.value, '_blank')
}

onMounted(() => fetchGA4Status())

watch(() => props.project.id, () => fetchGA4Status())
</script>

<template>
  <div class="funnels-view">
    <div class="d-flex align-center justify-space-between mb-8">
      <div>
        <h1 class="text-h4 font-weight-bold mb-1">Funnels</h1>
        <p class="text-subtitle-1 text-grey">Visualize conversion steps and identify drop-offs for {{ project.name }}</p>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="text-center py-12">
      <v-progress-circular indeterminate color="primary" size="48" />
      <p class="text-grey mt-4">Loading funnel data...</p>
    </div>

    <!-- GA4 Connected: Show Funnel -->
    <div v-else-if="isGA4Connected && ga4Status">
      <FunnelChart :project-id="project.id" :connector-id="ga4Status.connector_id" />
    </div>

    <!-- GA4 Not Connected: Prompt -->
    <v-card v-else class="empty-state-card pa-6 d-flex align-center" elevation="0">
      <v-avatar color="primary-lighten-5" size="48" class="mr-4">
        <v-icon icon="mdi-google-analytics" color="primary" size="24" />
      </v-avatar>
      <div class="flex-grow-1">
        <div class="text-subtitle-1 font-weight-bold">No data source connected</div>
        <p class="text-body-2 text-grey mb-0">Link your GA4 property in Project Settings to view funnels.</p>
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

.empty-state-card {
  border-radius: 24px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}
</style>
