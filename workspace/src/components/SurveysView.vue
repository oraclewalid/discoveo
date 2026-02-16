<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import Papa from 'papaparse'
import config from '@/config'
import qualitativeService from '@/services/qualitativeService'
import type { SurveyStats } from '@/types/analytics'

const props = defineProps<{
  projectId: string
}>()

const file = ref<File | null>(null)
const isUploading = ref(false)
const uploadStatus = ref<{ type: 'success' | 'error'; message: string } | null>(null)
const dragOver = ref(false)

const REQUIRED_HEADERS = [
  'Date',
  'Country',
  'URL',
  'Device',
  'Browser',
  'OS',
  'Ratings',
  'Comments'
]

const validateCSV = (selectedFile: File) => {
  return new Promise<{ isValid: boolean; error?: string }>((resolve) => {
    Papa.parse(selectedFile, {
      preview: 1, // Read only the first row (headers)
      header: false,
      skipEmptyLines: true,
      complete: (results) => {
        if (results.errors.length > 0) {
          resolve({ isValid: false, error: 'CSV parsing error. Please check the file formatting.' })
          return
        }

        const headers = results.data[0] as string[]
        if (!headers || headers.length === 0) {
          resolve({ isValid: false, error: 'The CSV file appears to be empty.' })
          return
        }

        // Check for missing headers
        const missingHeaders = REQUIRED_HEADERS.filter(
          (required) => !headers.some(h => h.trim().toLowerCase() === required.toLowerCase())
        )

        if (missingHeaders.length > 0) {
          resolve({ 
            isValid: false, 
            error: `Missing required columns: ${missingHeaders.join(', ')}` 
          })
        } else {
          resolve({ isValid: true })
        }
      },
      error: (err) => {
        resolve({ isValid: false, error: `Error reading file: ${err.message}` })
      }
    })
  })
}

const handleFileSelect = async (event: Event) => {
  const target = event.target as HTMLInputElement
  if (target.files && target.files.length > 0) {
    const selectedFile = target.files[0]
    await processFile(selectedFile)
  }
}

const handleDrop = async (event: DragEvent) => {
  dragOver.value = false
  if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
    const droppedFile = event.dataTransfer.files[0]
    await processFile(droppedFile)
  }
}

const processFile = async (selectedFile: File) => {
  uploadStatus.value = null
  
  if (selectedFile.type !== 'text/csv' && !selectedFile.name.endsWith('.csv')) {
    uploadStatus.value = { type: 'error', message: 'Please select a valid CSV file.' }
    file.value = null
    return
  }

  const validation = await validateCSV(selectedFile)
  if (!validation.isValid) {
    uploadStatus.value = { type: 'error', message: validation.error || 'Invalid CSV format.' }
    file.value = null
  } else {
    file.value = selectedFile
  }
}

const uploadFile = async () => {
  if (!file.value) return

  isUploading.value = true
  uploadStatus.value = null

  const formData = new FormData()
  formData.append('file', file.value)

  try {
    const response = await fetch(`${config.api.baseUrl}projects/${props.projectId}/qualitative/surveys`, {
      method: 'POST',
      body: formData,
      // Note: Don't set Content-Type header when using FormData, 
      // the browser will set it with the correct boundary
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => null)
      throw new Error(errorData?.error || errorData?.message || `Upload failed: ${response.statusText}`)
    }

    const result = await response.json()
    uploadStatus.value = { 
      type: 'success', 
      message: `Successfully uploaded ${result.row_count} survey rows!` 
    }
    file.value = null
  } catch (error) {
    console.error('Upload error:', error)
    uploadStatus.value = { 
      type: 'error', 
      message: error instanceof Error ? error.message : 'An error occurred during upload.' 
    }
  } finally {
    isUploading.value = false
  }
}

const stats = ref<SurveyStats | null>(null)
const isLoadingStats = ref(false)

const loadStats = async () => {
  isLoadingStats.value = true
  try {
    stats.value = await qualitativeService.getSurveyStats(props.projectId)
  } catch (error) {
    console.error('Failed to load survey stats:', error)
  } finally {
    isLoadingStats.value = false
  }
}

const clearFile = () => {
  file.value = null
  uploadStatus.value = null
}

const formatDate = (dateString?: string) => {
  if (!dateString) return '-'
  return new Date(dateString).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  })
}

onMounted(() => {
  loadStats()
})

watch(() => props.projectId, () => loadStats())
</script>

<template>
  <div class="surveys-view">
    <div class="mb-8">
      <h1 class="text-h3 font-weight-bold tracking-tight mb-2">Qualitative Data</h1>
      <p class="text-subtitle-1 text-grey-darken-1">Upload surveys, interviews, and usability test results</p>
    </div>

    <!-- Stats Summary Section -->
    <v-row v-if="stats" class="mb-8">
      <v-col cols="12" sm="6" md="3">
        <v-card class="stat-card" elevation="0">
          <v-card-text>
            <div class="d-flex align-center mb-2">
              <v-avatar color="primary-lighten-5" size="32" class="mr-2">
                <v-icon icon="mdi-poll" color="primary" size="18" />
              </v-avatar>
              <span class="text-caption font-weight-bold text-grey">TOTAL RESPONSES</span>
            </div>
            <div class="text-h4 font-weight-bold">{{ stats.total_responses.toLocaleString() }}</div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" sm="6" md="3">
        <v-card class="stat-card" elevation="0">
          <v-card-text>
            <div class="d-flex align-center mb-2">
              <v-avatar color="warning-lighten-5" size="32" class="mr-2">
                <v-icon icon="mdi-star" color="warning" size="18" />
              </v-avatar>
              <span class="text-caption font-weight-bold text-grey">AVG. RATING</span>
            </div>
            <div class="text-h4 font-weight-bold">
              {{ stats.average_rating.toFixed(2) }}
              <span class="text-body-2 text-grey font-weight-medium">/ 5</span>
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" sm="6" md="3">
        <v-card class="stat-card" elevation="0">
          <v-card-text>
            <div class="d-flex align-center mb-2">
              <v-avatar color="info-lighten-5" size="32" class="mr-2">
                <v-icon icon="mdi-comment-text-multiple" color="info" size="18" />
              </v-avatar>
              <span class="text-caption font-weight-bold text-grey">WITH COMMENTS</span>
            </div>
            <div class="text-h4 font-weight-bold">{{ stats.responses_with_comments.toLocaleString() }}</div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" sm="6" md="3">
        <v-card class="stat-card" elevation="0">
          <v-card-text>
            <div class="d-flex align-center mb-2">
              <v-avatar color="success-lighten-5" size="32" class="mr-2">
                <v-icon icon="mdi-calendar-range" color="success" size="18" />
              </v-avatar>
              <span class="text-caption font-weight-bold text-grey">DATE RANGE</span>
            </div>
            <div class="text-subtitle-1 font-weight-bold lh-1 mt-1">
              {{ formatDate(stats.first_response_date) }} -<br>
              {{ formatDate(stats.last_response_date) }}
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
        <v-card class="premium-upload-card" elevation="0">
          <v-card-text class="pa-10">
            <div 
              class="upload-zone"
              :class="{ 'drag-over': dragOver }"
              @dragover.prevent="dragOver = true"
              @dragleave.prevent="dragOver = false"
              @drop.prevent="handleDrop"
              @click="$refs.fileInput.click()"
            >
              <input 
                type="file" 
                ref="fileInput" 
                class="d-none" 
                accept=".csv"
                @change="handleFileSelect"
              >
              
              <v-avatar color="primary-lighten-5" size="100" class="mb-6">
                <v-icon icon="mdi-cloud-upload-outline" color="primary" size="48" />
              </v-avatar>

              <template v-if="!file">
                <h2 class="text-h5 font-weight-bold mb-2">Click or drag to upload</h2>
                <p class="text-body-1 text-grey">Support for .csv files only</p>
              </template>
              
              <template v-else>
                <div class="d-flex align-center bg-white pa-4 rounded-xl border mb-4 shadow-sm w-100">
                  <v-icon icon="mdi-file-delimited-outline" color="primary" class="mr-4" size="32" />
                  <div class="text-left flex-grow-1 overflow-hidden">
                    <div class="text-subtitle-1 font-weight-bold text-truncate">{{ file.name }}</div>
                    <div class="text-caption text-grey">{{ (file.size / 1024).toFixed(1) }} KB</div>
                  </div>
                  <v-btn icon="mdi-close" variant="text" size="small" color="grey" @click.stop="clearFile" />
                </div>
              </template>
            </div>

            <v-alert
              v-if="uploadStatus"
              :type="uploadStatus.type"
              variant="tonal"
              closable
              class="mt-6 rounded-xl"
              @click:close="uploadStatus = null"
            >
              {{ uploadStatus.message }}
            </v-alert>

            <div class="mt-8">
              <v-btn
                block
                color="primary"
                size="x-large"
                class="rounded-xl font-weight-bold elevation-4"
                :disabled="!file"
                :loading="isUploading"
                @click="uploadFile"
              >
                Upload Survey Data
              </v-btn>
            </div>
          </v-card-text>
        </v-card>

        <v-card class="mt-8 side-card" elevation="0">
          <v-card-title class="pa-6 border-bottom">
            <span class="text-h6 font-weight-bold">CSV Template Guide</span>
          </v-card-title>
          <v-card-text class="pa-6">
            <p class="text-body-2 text-grey-darken-1 mb-4">
              To ensure proper processing, please make sure your CSV includes the following headers:
            </p>
            <v-chip-group>
              <v-chip v-for="header in REQUIRED_HEADERS" :key="header" size="small" variant="tonal" border>{{ header }}</v-chip>
            </v-chip-group>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<style scoped>
.surveys-view {
  width: 100%;
}

.premium-upload-card {
  border-radius: 32px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}

.upload-zone {
  border: 2px dashed #e2e8f0;
  border-radius: 24px;
  padding: 60px 40px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s ease;
  background: #f8fafc;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.upload-zone:hover, .upload-zone.drag-over {
  border-color: #6366f1;
  background: #f1f5f9;
}

.upload-zone.drag-over {
  transform: scale(1.02);
}

.side-card {
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
}

.border-bottom {
  border-bottom: 1px solid #f1f5f9;
}

.tracking-tight {
  letter-spacing: -0.025em;
}

.stat-card {
  border-radius: 20px !important;
  border: 1px solid #e2e8f0 !important;
  background: white !important;
  height: 100%;
}

.lh-1 {
  line-height: 1.2 !important;
}

.text-grey-darken-1 {
  color: #475569 !important;
}
</style>
