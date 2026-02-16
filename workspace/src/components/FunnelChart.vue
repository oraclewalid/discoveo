<template>
  <div class="funnel-chart">
    <!-- Filters Card -->
    <v-card class="mb-6 filter-card" elevation="0">
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
              @update:model-value="handleFilterChange"
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
              @update:model-value="handleFilterChange"
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
              @update:model-value="handleFilterChange"
            />
          </v-col>

          <v-col cols="12" sm="2" class="text-right">
            <v-btn
              @click="loadFunnelData"
              :loading="isLoading"
              color="primary"
              variant="flat"
              block
              prepend-icon="mdi-refresh"
            >
              Update
            </v-btn>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>

    <!-- Content Tabs -->
    <v-card elevation="0" class="content-card">
      <v-tabs v-model="activeTab" color="primary" align-tabs="start" class="border-bottom">
        <v-tab value="viz" class="text-none font-weight-bold">
          <v-icon start icon="mdi-chart-finance" />
          Visualization
        </v-tab>
        <v-tab value="data" class="text-none font-weight-bold">
          <v-icon start icon="mdi-table" />
          Data Table
        </v-tab>
      </v-tabs>

      <v-card-text class="pa-0">
        <v-window v-model="activeTab">
          <!-- Visualization Tab -->
          <v-window-item value="viz">
            <div class="pa-8">
              <div v-if="isLoading" class="text-center py-12">
                <v-progress-circular indeterminate color="primary" size="48" />
                <p class="text-grey mt-4">Analyzing funnel trends...</p>
              </div>

              <div v-else-if="!funnelData || funnelData.length === 0" class="text-center py-12 text-grey">
                <v-icon icon="mdi-chart-arc" size="64" class="mb-4 opacity-20" />
                <p class="text-h6">No data to visualize</p>
                <p class="text-caption">Adjust your filters to see results</p>
              </div>

              <v-chart
                v-else
                :option="chartOption"
                style="width: 100%; height: 500px"
                autoresize
              />
            </div>
          </v-window-item>

          <!-- Data Table Tab -->
          <v-window-item value="data">
            <div class="pa-0">
              <v-data-table
                :headers="tableHeaders"
                :items="funnelData || []"
                :loading="isLoading"
                class="funnel-table"
                hover
              >
                <!-- Custom cell formatters -->
                <template #[`item.total_users`]="{ item }">
                  <span class="font-weight-bold">{{ item.total_users.toLocaleString() }}</span>
                </template>
                <template #[`item.total_interactions`]="{ item }">
                  {{ item.total_interactions.toLocaleString() }}
                </template>
                <template #[`item.users_dropped`]="{ item }">
                  <span class="text-error font-weight-medium">
                    {{ item.users_dropped !== null ? item.users_dropped.toLocaleString() : '-' }}
                  </span>
                </template>
                <template #[`item.dropoff_pct`]="{ item }">
                  <v-chip
                    v-if="item.dropoff_pct !== null"
                    :color="item.dropoff_pct < 0 ? 'success' : 'error'"
                    size="x-small"
                    variant="tonal"
                    label
                    class="font-weight-bold"
                  >
                    {{ item.dropoff_pct.toFixed(1) }}%
                  </v-chip>
                  <span v-else>-</span>
                </template>
                <template #[`item.conversion_from_start_pct`]="{ item }">
                  <div class="d-flex align-center">
                    <span class="mr-2">{{ item.conversion_from_start_pct.toFixed(1) }}%</span>
                    <v-progress-linear
                      :model-value="item.conversion_from_start_pct"
                      color="primary"
                      height="4"
                      rounded
                      style="width: 60px"
                    />
                  </div>
                </template>
                <template #[`item.stage_conversion_pct`]="{ item }">
                  <v-chip
                    v-if="item.stage_conversion_pct !== null"
                    color="primary"
                    size="x-small"
                    variant="outlined"
                  >
                    {{ item.stage_conversion_pct.toFixed(1) }}%
                  </v-chip>
                  <span v-else>-</span>
                </template>
              </v-data-table>
            </div>
          </v-window-item>
        </v-window>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import ga4FunnelService from '@/services/ga4FunnelService'
import type { FunnelData } from '@/types/funnel'

interface Props {
  projectId: string
  connectorId: string
  dimensionOptions?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  dimensionOptions: () => ['device_category', 'country', 'browser', 'operating_system'],
})

// State
const funnelData = ref<FunnelData | null>(null)
const isLoading = ref(false)
const error = ref('')
const activeTab = ref('viz')

// Filters
const startDate = ref<string>('')
const endDate = ref<string>('')
const selectedDimension = ref('All')

const dimensions = computed(() => ['All', ...props.dimensionOptions])

// Table headers
const tableHeaders = computed(() => {
  const headers: any[] = [
    { title: 'Rank', key: 'stage_order', width: '70px', align: 'center' },
    { title: 'Funnel Stage', key: 'funnel_stage', align: 'start' },
  ]

  if (selectedDimension.value !== 'All') {
    headers.push({ title: 'Segment', key: 'dimension', align: 'start' })
  }

  headers.push(
    { title: 'Users', key: 'total_users', align: 'start' },
    { title: 'Interactions', key: 'total_interactions', align: 'start' },
    { title: 'Dropped', key: 'users_dropped', align: 'start' },
    { title: 'Drop Rate', key: 'dropoff_pct', align: 'center' },
    { title: 'Funnel Progress', key: 'conversion_from_start_pct', align: 'start' },
    { title: 'Conversion', key: 'stage_conversion_pct', align: 'center' },
  )

  return headers
})

// Initialize dates
onMounted(() => {
  const end = new Date()
  const start = new Date(end.getTime() - 30 * 24 * 60 * 60 * 1000)
  endDate.value = end.toISOString().split('T')[0]!
  startDate.value = start.toISOString().split('T')[0]!
  loadFunnelData()
})

const formatDateForAPI = (dateString: string): string => dateString.replace(/-/g, '')

const loadFunnelData = async () => {
  if (!startDate.value || !endDate.value) return
  isLoading.value = true
  error.value = ''
  try {
    const dimension = selectedDimension.value === 'All' ? 'all' : selectedDimension.value
    const data = await ga4FunnelService.getFunnelData(
      props.projectId,
      props.connectorId,
      dimension,
      formatDateForAPI(startDate.value),
      formatDateForAPI(endDate.value),
    )
    funnelData.value = data
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load'
  } finally {
    isLoading.value = false
  }
}

const handleFilterChange = () => {}

// Chart option computed
const chartOption = computed(() => {
  if (!funnelData.value || funnelData.value.length === 0) return {}
  const data = funnelData.value
  const stageMap = new Map<number, Map<string, number>>()
  const stageNameMap = new Map<number, string>()
  const dimensionSet = new Set<string>()

  data.forEach((item) => {
    stageNameMap.set(item.stage_order, item.funnel_stage)
    dimensionSet.add(item.dimension)
    if (!stageMap.has(item.stage_order)) stageMap.set(item.stage_order, new Map())
    stageMap.get(item.stage_order)!.set(item.dimension, item.total_users)
  })

  const stages = Array.from(stageMap.keys()).sort((a, b) => a - b)
  const stageLabels = stages.map((order) => stageNameMap.get(order) || '')
  const dimensions = Array.from(dimensionSet).sort()

  const series = dimensions.map((dimValue) => ({
    name: dimValue,
    type: 'bar',
    stack: 'total',
    barWidth: '60%',
    data: stages.map((stageOrder) => stageMap.get(stageOrder)?.get(dimValue) || 0),
    itemStyle: { borderRadius: 4 }
  }))

  return {
    color: ['#6366f1', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981', '#3b82f6'],
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(255, 255, 255, 0.95)',
      borderColor: '#e2e8f0',
      borderWidth: 1,
      textStyle: { color: '#1e293b' },
      axisPointer: { type: 'shadow' }
    },
    legend: { bottom: 0, icon: 'circle', itemGap: 20 },
    grid: { left: '2%', right: '2%', bottom: '10%', top: '5%', containLabel: true },
    xAxis: {
      type: 'category',
      data: stageLabels,
      axisLine: { lineStyle: { color: '#e2e8f0' } },
      axisLabel: { color: '#64748b', fontSize: 11 }
    },
    yAxis: {
      type: 'value',
      splitLine: { lineStyle: { type: 'dashed', color: '#f1f5f9' } },
      axisLabel: { color: '#64748b' }
    },
    series: series as any,
  }
})
</script>

<style scoped>
.funnel-chart {
  width: 100%;
}

.filter-card {
  background: #f8fafc !important;
  border-radius: 16px !important;
  border: 1px solid #f1f5f9 !important;
}

.custom-field :deep(.v-field) {
  background: white !important;
  border-radius: 10px !important;
  border: 1px solid #e2e8f0 !important;
}

.content-card {
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
  overflow: hidden;
}

.border-bottom {
  border-bottom: 1px solid #f1f5f9;
}

.funnel-table {
  background: transparent !important;
}

.funnel-table :deep(th) {
  background: #f8fafc !important;
  text-transform: uppercase;
  font-size: 11px !important;
  letter-spacing: 0.05em;
  font-weight: 700 !important;
  color: #64748b !important;
}
</style>
