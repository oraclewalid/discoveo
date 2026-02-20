<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Project } from '@/types/project'
import croService from '@/services/croService'

const props = defineProps<{
  project: Project
  reportId?: string | null
}>()

const loading = ref(false)
const lastReport = ref<any>(null)
const reportsHistory = ref<any[]>([])
const error = ref('')

const generateReport = async () => {
  try {
    loading.value = true
    error.value = ''

    // Generate new report
    await croService.generateReport(props.project.id)

    // Emit event so parent can refresh history
    emit('generated')
    // After generation, we usually want to show the latest
    await loadLatestReport()
  } catch (err) {
    error.value = 'Failed to generate report'
    console.error(err)
  } finally {
    loading.value = false
  }
}

const emit = defineEmits(['generated', 'select-report'])

const loadReports = async () => {
  try {
    loading.value = true
    const reports = await croService.listReports(props.project.id)
    reportsHistory.value = reports.sort((a: any, b: any) => 
      new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    )
  } catch (err) {
    console.error('Failed to load reports:', err)
  } finally {
    loading.value = false
  }
}

const loadReportDetails = async (reportId: string) => {
  try {
    loading.value = true
    lastReport.value = await croService.getReport(props.project.id, reportId)
    // Scroll to top of report
    window.scrollTo({ top: 0, behavior: 'smooth' })
  } catch (err) {
    error.value = 'Failed to load report details'
    console.error(err)
  } finally {
    loading.value = false
  }
}

const loadLatestReport = async () => {
  try {
    loading.value = true
    const reports = await croService.listReports(props.project.id)
    if (reports && reports.length > 0) {
      const latest = reports.sort((a: any, b: any) => 
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
      )[0]
      if (latest && latest.id) {
        await loadReportDetails(latest.id)
      }
    } else {
      lastReport.value = null
    }
  } catch (err) {
    console.error('Failed to load latest report:', err)
  } finally {
    loading.value = false
  }
}

watch(() => props.reportId, (newId) => {
  if (newId === 'history') {
    lastReport.value = null
    loadReports()
  } else if (newId) {
    loadReportDetails(newId)
  } else {
    loadLatestReport()
  }
}, { immediate: true })

/* Helper Functions */
const getPriorityColor = (priority: string) => {
  switch (priority?.toLowerCase()) {
    case 'high': return 'error'
    case 'medium': return 'warning'
    case 'low': return 'info'
    default: return 'grey'
  }
}

const getSeverityColor = (severity: string) => {
  switch (severity?.toLowerCase()) {
    case 'critical': return 'error'
    case 'major': return 'warning'
    case 'minor': return 'info'
    default: return 'grey'
  }
}

const getSentimentColor = (sentiment: string) => {
  switch (sentiment?.toLowerCase()) {
    case 'positive': return 'success'
    case 'negative': return 'error'
    case 'mixed': return 'warning'
    default: return 'grey'
  }
}

const formatDate = (dateStr: string) => {
  if (!dateStr) return ''
  return new Date(dateStr).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}
</script>

<template>
  <div class="ai-view">
    <div class="mb-8">
      <h1 class="text-h4 font-weight-bold mb-1">AI Recommendation</h1>
      <p class="text-subtitle-1 text-grey">
        CRO Audit and Conversion Roadmap for {{ project.name }}
      </p>
    </div>

    <v-card class="pa-6" elevation="2">
      <div class="d-flex align-center justify-space-between mb-6">
        <div>
          <h2 class="text-h5 font-weight-bold">
            {{ props.reportId === 'history' ? 'Audit History' : 'CRO Audit Report' }}
          </h2>
          <p class="text-subtitle-2 text-grey">
            <template v-if="props.reportId === 'history'">
              Detailed history of all conversion optimization reports.
            </template>
            <template v-else>
              {{ lastReport ? 'Report from ' + formatDate(lastReport.createdAt) : 'Click to generate a new analysis' }}
            </template>
          </p>
        </div>
        <div class="d-flex gap-2">
          <v-btn
            v-if="props.reportId === 'history'"
            variant="outlined"
            prepend-icon="mdi-arrow-left"
            @click="emit('select-report', null)"
            class="mr-2"
          >
            Back
          </v-btn>
          <v-btn
            color="primary"
            size="large"
            prepend-icon="mdi-sparkles"
            @click="generateReport"
            :loading="loading"
            :disabled="loading"
          >
            Generate New Audit
          </v-btn>
        </div>
      </div>

      <v-divider class="my-6"></v-divider>

      <v-alert v-if="error" type="error" class="mb-6" variant="tonal">
        {{ error }}
      </v-alert>

      <!-- Loading State -->
      <div v-if="loading && !lastReport && props.reportId !== 'history'" class="text-center py-12">
        <v-progress-circular indeterminate color="primary" size="64" />
        <div class="mt-4 text-grey">Analyzing project data...</div>
      </div>

      <!-- History List View -->
      <div v-else-if="props.reportId === 'history'" class="history-list mt-6">
        <v-list v-if="reportsHistory.length > 0" lines="three" class="bg-transparent pa-0">
          <v-list-item
            v-for="report in reportsHistory"
            :key="report.id"
            @click="emit('select-report', report.id)"
            class="mb-4 rounded-xl border-light bg-white history-item pa-4"
            elevation="1"
          >
            <template v-slot:prepend>
              <v-avatar color="primary-lighten-5" size="48" class="mr-4">
                <v-icon icon="mdi-file-clock-outline" color="primary"></v-icon>
              </v-avatar>
            </template>
            <v-list-item-title class="text-h6 font-weight-bold mb-1">
              Analysis from {{ formatDate(report.createdAt) }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-body-2 text-grey-darken-1 line-clamp-2">
              {{ report.executive_summary || report.analysis?.executive_summary || 'Analysis complete. View full report for detailed conversion insights and roadmap.' }}
            </v-list-item-subtitle>
            <template v-slot:append>
              <v-btn variant="tonal" icon="mdi-chevron-right" color="primary" class="rounded-lg ml-2"></v-btn>
            </template>
          </v-list-item>
        </v-list>
        <div v-else-if="!loading" class="text-center py-12">
          <v-icon icon="mdi-history" size="64" color="grey-lighten-1" class="mb-4" />
          <p class="text-grey">No reports found for this project yet.</p>
        </div>
        <div v-else class="text-center py-12">
          <v-progress-circular indeterminate color="primary" />
        </div>
      </div>

      <!-- Report Detail View -->
      <div v-else-if="lastReport" class="report-content mt-6">
        <!-- Executive Summary -->
        <v-card class="narrative-card mb-8 px-6 py-4" elevation="0">
          <v-row align="center">
            <v-col cols="12" md="1" class="text-center">
              <v-avatar color="primary-lighten-5" size="64">
                <v-icon icon="mdi-robot-outline" color="primary" size="32" />
              </v-avatar>
            </v-col>
            <v-col cols="12" md="11">
              <h3 class="text-h6 font-weight-bold mb-1">Executive Summary</h3>
              <p class="text-body-1 text-grey-darken-3 mb-0 italic-narrative">
                {{ lastReport.executive_summary || lastReport.analysis?.executive_summary }}
              </p>
            </v-col>
          </v-row>
        </v-card>

        <!-- Funnel Analysis Section -->
        <section class="mb-10">
          <h3 class="text-h5 font-weight-bold mb-4 d-flex align-center">
            <v-icon icon="mdi-filter-variant" color="secondary" class="mr-2" />
            Funnel Analysis
          </h3>
          <v-card variant="outlined" class="pa-6 border-light rounded-xl mb-4 bg-grey-lighten-5">
            <p class="text-body-2 mb-0">{{ lastReport.funnel_analysis?.overview }}</p>
          </v-card>
          
          <v-row v-if="lastReport.funnel_analysis?.critical_drop_offs?.length">
            <v-col v-for="dropoff in lastReport.funnel_analysis.critical_drop_offs" :key="dropoff.stage" cols="12" md="4">
              <v-card class="dropoff-card h-100" elevation="0">
                <div :class="`severity-bar bg-${getSeverityColor(dropoff.severity)}`"></div>
                <v-card-text class="pa-4">
                  <div class="d-flex justify-space-between align-start mb-2">
                    <div class="text-caption font-weight-bold text-grey text-uppercase">{{ dropoff.stage }}</div>
                    <v-chip :color="getSeverityColor(dropoff.severity)" size="x-small" label class="font-weight-bold">
                      -{{ dropoff.drop_rate }}%
                    </v-chip>
                  </div>
                  <div class="text-body-2 italic text-grey-darken-2">
                    "{{ dropoff.correlated_feedback?.[0] }}"
                  </div>
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>
        </section>

        <!-- Qualitative Insights Section -->
        <section class="mb-10">
          <h3 class="text-h5 font-weight-bold mb-4 d-flex align-center">
            <v-icon icon="mdi-comment-quote-outline" color="primary" class="mr-2" />
            Qualitative Insights
          </h3>
          <p class="text-body-2 text-grey-darken-1 mb-6">{{ lastReport.qualitative_insights?.overview }}</p>
          
          <v-row>
            <v-col v-for="theme in lastReport.qualitative_insights?.themes_with_data" :key="theme.theme" cols="12" md="6">
              <v-card class="theme-card h-100" elevation="0">
                <v-card-item>
                  <template v-slot:prepend>
                    <v-icon 
                      :icon="theme.sentiment === 'positive' ? 'mdi-emoticon-happy-outline' : 'mdi-alert-circle-outline'"
                      :color="getSentimentColor(theme.sentiment)"
                      class="mr-2"
                    />
                  </template>
                  <v-card-title class="text-subtitle-1 font-weight-bold">{{ theme.theme }}</v-card-title>
                </v-card-item>
                <v-card-text>
                  <div class="quotes-container mb-4">
                    <div v-for="(quote, i) in theme.supporting_quotes" :key="i" class="quote text-caption text-grey-darken-2">
                      "{{ quote }}"
                    </div>
                  </div>
                  <div class="metrics-grid">
                    <div v-for="(metric, i) in theme.related_metrics" :key="i" class="metric-item">
                      <v-icon icon="mdi-chart-line" size="14" class="mr-1" />
                      {{ metric }}
                    </div>
                  </div>
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>
        </section>

        <!-- Recommendations Section -->
        <section>
          <h3 class="text-h5 font-weight-bold mb-6 d-flex align-center">
            <v-icon icon="mdi-lightbulb-on-outline" color="warning" class="mr-2" />
            Conversion Optimization Roadmap
          </h3>
          <v-row>
            <v-col v-for="rec in lastReport.recommendations" :key="rec.title" cols="12">
              <v-card class="recommendation-card overflow-hidden" elevation="0">
                <v-row no-gutters>
                  <v-col cols="12" md="8" class="pa-6">
                    <div class="d-flex align-center mb-3">
                      <v-chip :color="getPriorityColor(rec.priority)" size="small" class="mr-3 font-weight-bold">
                        {{ rec.priority.toUpperCase() }}
                      </v-chip>
                      <v-chip variant="outlined" size="small" class="mr-3">{{ rec.category }}</v-chip>
                      <h4 class="text-h6 font-weight-bold">{{ rec.title }}</h4>
                    </div>
                    <p class="text-body-2 mb-4">{{ rec.description }}</p>
                    <div class="evidence-box pa-3 rounded bg-grey-lighten-4">
                      <div class="text-caption font-weight-bold text-grey mb-2 uppercase tracking-wide">Supporting Data</div>
                      <div v-for="(ev, i) in rec.supporting_evidence" :key="i" class="text-caption d-flex align-start mb-1">
                        <v-icon icon="mdi-check-circle" color="grey" size="12" class="mt-1 mr-2" />
                        {{ ev }}
                      </div>
                    </div>
                  </v-col>
                  <v-col cols="12" md="4" class="pa-6 bg-primary-lighten-5 d-flex flex-column justify-center border-left">
                    <div class="text-overline font-weight-bold text-primary mb-1">Impact Goal</div>
                    <div class="text-body-1 font-weight-bold text-primary-darken-2">
                      {{ rec.expected_impact }}
                    </div>
                  </v-col>
                </v-row>
              </v-card>
            </v-col>
          </v-row>
        </section>
      </div>

      <div v-else class="text-center py-12">
        <v-avatar color="grey-lighten-4" size="120" class="mb-6">
          <v-icon icon="mdi-chart-box-plus-outline" size="64" color="grey"></v-icon>
        </v-avatar>
        <h2 class="text-h5 font-weight-bold mb-2">Ready for Analysis?</h2>
        <p class="text-subtitle-1 text-grey max-w-500 mx-auto">
          Our AI will audit your site-wide data, funnels, and qualitative feedback to build your conversion roadmap.
        </p>
      </div>
    </v-card>
  </div>
</template>

<style scoped>
.ai-view {
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
}

.narrative-card {
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
  border-left: 6px solid #6366f1 !important;
}

.italic-narrative {
  font-style: italic;
  line-height: 1.6;
}

.dropoff-card {
  border-radius: 16px !important;
  border: 1px solid #e2e8f0 !important;
  position: relative;
  overflow: hidden;
}

.severity-bar {
  height: 4px;
  width: 100%;
  position: absolute;
  top: 0;
}

.theme-card {
  border-radius: 16px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}

.quotes-container {
  border-left: 2px solid #f1f5f9;
  padding-left: 12px;
}

.quote {
  font-style: italic;
  margin-bottom: 6px;
  line-height: 1.4;
}

.metric-item {
  font-size: 0.75rem;
  color: #64748b;
  background: #f8fafc;
  padding: 4px 10px;
  border-radius: 6px;
  margin-bottom: 4px;
  display: inline-block;
  margin-right: 6px;
}

.recommendation-card {
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
}

.border-left {
  border-left: 1px solid #e2e8f0 !important;
}

.max-w-500 {
  max-width: 500px;
}

.gap-2 {
  gap: 8px;
}

.history-item {
  transition: all 0.2s ease;
  border: 1px solid #f1f5f9 !important;
}

.history-item:hover {
  background-color: #f8fafc !important;
  transform: translateY(-2px);
  border-color: #6366f1 !important;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (max-width: 960px) {
  .border-left {
    border-left: none !important;
    border-top: 1px solid #e2e8f0 !important;
  }
}
</style>
