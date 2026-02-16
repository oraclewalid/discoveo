<template>
  <div class="scroll-depth-analysis">
    <!-- Filters Card (Hidden if externally controlled) -->
    <v-card v-if="!isExternallyControlled" class="mb-6 filter-card" elevation="0">
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
              @update:model-value="loadData"
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
              @update:model-value="loadData"
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
              @update:model-value="loadData"
            />
          </v-col>

          <v-col cols="12" sm="2" class="text-right">
            <v-btn
              @click="loadData"
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

    <v-card elevation="0" class="content-card">
      <v-card-title class="px-8 pt-8 d-flex align-center">
        <v-icon icon="mdi-mouse-move" color="primary" class="mr-3" />
        Scroll Depth Analysis
        <v-spacer />
        <v-btn
          icon="mdi-refresh"
          variant="text"
          size="small"
          :loading="isLoading"
          @click="loadData"
        />
      </v-card-title>

      <!-- Tabs Header -->
      <v-tabs v-model="activeTab" color="primary" align-tabs="start" class="px-4 border-bottom">
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
                <p class="text-grey mt-4">Analyzing reader engagement...</p>
              </div>

              <div v-else-if="error" class="text-center py-12">
                <v-icon icon="mdi-alert-circle-outline" color="error" size="48" class="mb-4" />
                <p class="text-error font-weight-bold">{{ error }}</p>
                <v-btn variant="text" color="primary" class="mt-2" @click="loadData">Try again</v-btn>
              </div>

              <div v-else-if="!data || data.length === 0" class="text-center py-12 text-grey">
                <v-icon icon="mdi-gauge-empty" size="64" class="mb-4 opacity-20" />
                <p class="text-h6">No scroll data available</p>
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
                :headers="headers"
                :items="data"
                :loading="isLoading"
                class="scroll-table"
                hide-default-footer
                hover
              >
                <template #[`item.scroll_depth`]="{ item }">
                  <span class="font-weight-bold">{{ item.scroll_depth }}%</span>
                </template>
                <template #[`item.users`]="{ item }">
                  <span class="font-weight-bold">{{ item.users.toLocaleString() }}</span>
                </template>
                <template #[`item.events`]="{ item }">
                  {{ item.events.toLocaleString() }}
                </template>
                <template #[`item.drop_off_pct`]="{ item }">
                  <v-chip
                    v-if="item.drop_off_pct"
                    color="error"
                    size="x-small"
                    variant="tonal"
                    label
                    class="font-weight-bold"
                  >
                    {{ item.drop_off_pct }}%
                  </v-chip>
                  <span v-else class="text-grey opacity-50">-</span>
                </template>
                <template #[`item.users_lost`]="{ item }">
                  <span v-if="item.users_lost" class="text-error-darken-1 font-weight-bold">
                    -{{ item.users_lost.toLocaleString() }}
                  </span>
                  <span v-else class="text-grey opacity-50">-</span>
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
import { ref, onMounted, computed, watch } from 'vue'
import ga4FunnelService from '@/services/ga4FunnelService'
import type { ScrollDepthData } from '@/types/analytics'

interface Props {
  projectId: string
  connectorId: string
  dimensionOptions?: string[]
  externalStartDate?: string
  externalEndDate?: string
  externalDimension?: string
}

const props = withDefaults(defineProps<Props>(), {
  dimensionOptions: () => ['device_category', 'country', 'browser', 'operating_system'],
})

const data = ref<ScrollDepthData[]>([])
const isLoading = ref(false)
const error = ref('')
const activeTab = ref('viz')

// Filters
const startDate = ref<string>('')
const endDate = ref<string>('')
const selectedDimension = ref('All')

const dimensions = computed(() => ['All', ...props.dimensionOptions])

const isExternallyControlled = computed(() => 
  !!(props.externalStartDate || props.externalEndDate || props.externalDimension)
)

// Sync with props if externally controlled
watch(() => props.externalStartDate, (val) => { if (val) startDate.value = val })
watch(() => props.externalEndDate, (val) => { if (val) endDate.value = val })
watch(() => props.externalDimension, (val) => { if (val) selectedDimension.value = val })

// Watch for changes to trigger refresh
watch([() => props.externalStartDate, () => props.externalEndDate, () => props.externalDimension], () => {
  if (isExternallyControlled.value) loadData()
})

const headers = [
  { title: 'Depth', key: 'scroll_depth', align: 'start' as const },
  { title: 'Readers', key: 'users', align: 'end' as const },
  { title: 'Events', key: 'events', align: 'end' as const },
  { title: 'Users Lost', key: 'users_lost', align: 'end' as const },
  { title: 'Drop Rate', key: 'drop_off_pct', align: 'center' as const },
]

const formatDateForAPI = (dateString: string): string => dateString.replace(/-/g, '')

const loadData = async () => {
  if (!startDate.value || !endDate.value) return
  isLoading.value = true
  error.value = ''
  try {
    const dimension = selectedDimension.value === 'All' ? 'all' : selectedDimension.value
    const response = await ga4FunnelService.getScrollDepth(
      props.projectId,
      props.connectorId,
      dimension,
      formatDateForAPI(startDate.value),
      formatDateForAPI(endDate.value)
    )
    // Sort by scroll depth ascending (25, 50, 75, 90)
    data.value = (response as ScrollDepthData[]).sort((a, b) => a.scroll_depth - b.scroll_depth)
  } catch (err) {
    console.error('Failed to load scroll depth:', err)
    error.value = 'Failed to load scroll analysis'
  } finally {
    isLoading.value = false
  }
}

// Chart option computed
const chartOption = computed(() => {
  if (!data.value || data.value.length === 0) return {}
  
  // Sort data by depth ascending
  const sortedData = [...data.value].sort((a, b) => a.scroll_depth - b.scroll_depth)
  
  // Determine categories (Depths) and dimensions
  const depths = Array.from(new Set(sortedData.map(i => i.scroll_depth))).sort((a, b) => a - b)
  const depthLabels = depths.map(d => `${d}% Depth`)
  
  const dimensionSet = new Set<string>()
  const depthDimensionMap = new Map<number, Map<string, number>>()
  
  sortedData.forEach(item => {
    const dim = item.dimension || (selectedDimension.value === 'All' ? 'Total' : selectedDimension.value)
    dimensionSet.add(dim)
    if (!depthDimensionMap.has(item.scroll_depth)) {
      depthDimensionMap.set(item.scroll_depth, new Map())
    }
    depthDimensionMap.get(item.scroll_depth)!.set(dim, item.users)
  })
  
  const dimensions = Array.from(dimensionSet).sort()
  
  const series = dimensions.map(dimValue => ({
    name: dimValue,
    type: 'bar',
    stack: 'total',
    barWidth: '60%',
    data: depths.map(d => depthDimensionMap.get(d)?.get(dimValue) || 0),
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
    grid: { left: '2%', right: '2%', bottom: '10%', top: '10%', containLabel: true },
    xAxis: {
      type: 'category',
      data: depthLabels,
      axisLine: { lineStyle: { color: '#e2e8f0' } },
      axisLabel: { color: '#64748b', fontSize: 11 }
    },
    yAxis: {
      type: 'value',
      name: 'Readers',
      splitLine: { lineStyle: { type: 'dashed', color: '#f1f5f9' } },
      axisLabel: { color: '#64748b' }
    },
    series: series as any,
  }
})

onMounted(() => {
  if (isExternallyControlled.value) {
    if (props.externalStartDate) startDate.value = props.externalStartDate
    if (props.externalEndDate) endDate.value = props.externalEndDate
    if (props.externalDimension) selectedDimension.value = props.externalDimension
  } else {
    const end = new Date()
    const start = new Date(end.getTime() - 30 * 24 * 60 * 60 * 1000)
    endDate.value = end.toISOString().split('T')[0]!
    startDate.value = start.toISOString().split('T')[0]!
  }
  loadData()
})
</script>

<style scoped>
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
  border-radius: 24px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
  overflow: hidden;
}

.border-bottom {
  border-bottom: 1px solid #f1f5f9;
}

.scroll-table {
  background: transparent !important;
}

.scroll-table :deep(th) {
  background: #f8fafc !important;
  text-transform: uppercase;
  font-size: 11px !important;
  letter-spacing: 0.05em;
  font-weight: 700 !important;
  color: #64748b !important;
}

.scroll-table :deep(td) {
  padding: 16px !important;
}
</style>
