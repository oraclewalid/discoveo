<template>
  <div class="page-paths-table mt-8">
    <v-card elevation="0" class="content-card">
      <v-card-title class="px-8 pt-8 d-flex align-center">
        <v-icon icon="mdi-file-tree" color="primary" class="mr-3" />
        Popular Page Paths
        <v-spacer />
        <v-btn
          icon="mdi-refresh"
          variant="text"
          size="small"
          :loading="isLoading"
          @click="loadData"
        />
      </v-card-title>

      <v-card-text class="pa-0">
        <v-data-table
          :headers="headers"
          :items="data"
          :loading="isLoading"
          class="custom-table"
          hover
          :items-per-page="10"
        >
          <template #[`item.page_path`]="{ item }">
            <div class="d-flex align-center py-2">
              <v-icon icon="mdi-link" size="14" class="mr-2 text-grey-lighten-1" />
              <span class="font-weight-medium text-primary text-truncate" style="max-width: 300px;">
                {{ item.page_path }}
              </span>
            </div>
          </template>
          
          <template #[`item.total_pageviews`]="{ item }">
            <span class="font-weight-medium text-grey-darken-3">{{ item.total_pageviews.toLocaleString() }}</span>
          </template>

          <template #[`item.total_users`]="{ item }">
            <span class="font-weight-bold text-indigo-darken-2">{{ item.total_users.toLocaleString() }}</span>
          </template>

          <template #[`item.avg_time_per_pageview_sec`]="{ item }">
            <v-chip size="x-small" variant="tonal" color="info" class="font-weight-bold">
              {{ formatDuration(item.avg_time_per_pageview_sec) }}
            </v-chip>
          </template>

          <template #[`item.avg_time_per_user_sec`]="{ item }">
            <span class="text-caption text-grey">{{ formatDuration(item.avg_time_per_user_sec) }}</span>
          </template>
          
          <template #[`no-data`]>
            <div class="text-center py-12 text-grey">
              <v-icon icon="mdi-database-off" size="48" class="mb-4 opacity-20" />
              <p>No page path data found for this period</p>
            </div>
          </template>
        </v-data-table>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import ga4FunnelService from '@/services/ga4FunnelService'
import type { PagePathData } from '@/types/analytics'

interface Props {
  projectId: string
  connectorId: string
  externalStartDate?: string
  externalEndDate?: string
  externalDimension?: string
}

const props = defineProps<Props>()

const data = ref<PagePathData[]>([])
const isLoading = ref(false)
const error = ref('')

// Correctly mapped headers for the data table
const headers = [
  { title: 'Page Path', key: 'page_path', align: 'start' as const },
  { title: 'Views', key: 'total_pageviews', align: 'end' as const },
  { title: 'Users', key: 'total_users', align: 'end' as const },
  { title: 'Avg. Time/View', key: 'avg_time_per_pageview_sec', align: 'end' as const },
  { title: 'Avg. Session', key: 'avg_time_per_user_sec', align: 'end' as const },
]

const formatDateForAPI = (dateString: string): string => dateString.replace(/-/g, '')

const formatDuration = (seconds: number): string => {
  if (typeof seconds !== 'number') return '0s'
  if (seconds < 60) return `${seconds.toFixed(1)}s`
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = Math.floor(seconds % 60)
  return `${minutes}m ${remainingSeconds}s`
}

const loadData = async () => {
  if (!props.externalStartDate || !props.externalEndDate) return
  
  isLoading.value = true
  error.value = ''
  try {
    const dimension = props.externalDimension === 'All' ? 'all' : props.externalDimension || 'all'
    const response = await ga4FunnelService.getPagePaths(
      props.projectId,
      props.connectorId,
      dimension,
      formatDateForAPI(props.externalStartDate),
      formatDateForAPI(props.externalEndDate)
    )
    
    // Response can be direct array or { data: [...] }
    const pageData = Array.isArray(response) ? response : (response as any).data || []
    data.value = pageData as PagePathData[]
  } catch (err) {
    console.error('Failed to load page paths:', err)
    error.value = 'Failed to load page path data'
  } finally {
    isLoading.value = false
  }
}

// Watch for filter changes
watch(
  [() => props.externalStartDate, () => props.externalEndDate, () => props.externalDimension],
  () => loadData()
)

onMounted(() => loadData())
</script>

<style scoped>
.content-card {
  border-radius: 24px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
  overflow: hidden;
}

.custom-table :deep(th) {
  background: #f8fafc !important;
  text-transform: uppercase;
  font-size: 11px !important;
  letter-spacing: 0.05em;
  font-weight: 700 !important;
  color: #64748b !important;
  height: 48px !important;
}

.custom-table :deep(td) {
  padding: 12px 16px !important;
  border-bottom: 1px solid #f1f5f9 !important;
}

.custom-table :deep(.v-data-table-footer) {
  border-top: 1px solid #f1f5f9;
  background: #f8fafc;
}
</style>
