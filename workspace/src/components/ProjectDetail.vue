<template>
  <div class="project-detail">
    <button @click="handleBack" class="btn btn-back">
      &larr; Back to Projects
    </button>

    <div v-if="project" class="detail-container">
      <div class="detail-header">
        <h1>{{ project.name }}</h1>
        <div class="detail-id">
          <span>ID:</span>
          <code>{{ project.id }}</code>
        </div>
      </div>

      <div class="detail-content">
        <section class="description-section">
          <h2>Description</h2>
          <p v-if="project.description" class="description">
            {{ project.description }}
          </p>
          <p v-else class="no-description">No description provided</p>
        </section>

        <section class="connectors-section">
          <h2>Connectors</h2>

          <div v-if="isLoadingGA4" class="connector-loading">
            Loading GA4 status...
          </div>

          <div v-else-if="isGA4Connected && ga4Status" class="connector-connected">
            <div class="connection-header">
              <span :class="['status-indicator', ga4Status.is_expired ? 'expired' : 'connected']">●</span>
              <h3>Google Analytics 4 - {{ ga4Status.is_expired ? 'Expired' : 'Connected' }}</h3>
            </div>

            <div class="connection-details">
              <div class="detail-item">
                <span class="label">Connector ID:</span>
                <span class="value">{{ ga4Status.connector_id }}</span>
              </div>
              <div class="detail-item">
                <span class="label">Expires At:</span>
                <span class="value">{{ new Date(ga4Status.expires_at).toLocaleDateString() }}</span>
              </div>
              <div v-if="ga4Status.propertyName" class="detail-item">
                <span class="label">Property:</span>
                <span class="value">{{ ga4Status.propertyName }}</span>
              </div>
              <div v-if="ga4Status.lastSync" class="detail-item">
                <span class="label">Last Sync:</span>
                <span class="value">{{ new Date(ga4Status.lastSync).toLocaleDateString() }}</span>
              </div>
            </div>

            <div class="connector-actions">
              <button @click="handlePullData" class="btn btn-sm btn-primary">
                Pull Data
              </button>
              <button @click="handleDisconnectGA4" class="btn btn-sm btn-danger">
                Disconnect
              </button>
            </div>
          </div>

          <div v-else class="connector-list">
            <a
              :href="ga4AuthUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="connector-link"
            >
              <span class="connector-icon">📊</span>
              <span class="connector-name">Connect to GA4</span>
              <span class="external-icon">↗</span>
            </a>
          </div>

          <div v-if="ga4Error" class="error-message">
            {{ ga4Error }}
          </div>
        </section>
      </div>
    </div>

    <div v-else class="loading">
      Loading project details...
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import type { Project } from '@/types/project';
import type { GA4ConnectorStatus } from '@/types/ga4';
import projectService from '@/services/projectService';
import ga4Service from '@/services/ga4Service';
import config from '@/config';

interface Props {
  project: Project;
}

interface Emits {
  (e: 'back'): void;
  (e: 'edit'): void;
  (e: 'delete'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const project = computed(() => props.project);
const ga4Status = ref<GA4ConnectorStatus | null>(null);
const isLoadingGA4 = ref(false);
const ga4Error = ref('');

const ga4AuthUrl = computed(() => {
  return `${config.api.baseUrl}projects/${project.value.id}/connectors/ga4/auth/redirect`;
});

const isGA4Connected = computed(() => {
  return ga4Status.value !== null;
});

const handleBack = () => {
  emit('back');
};

const handleEdit = () => {
  emit('edit');
};

const handleDelete = async () => {
  if (!confirm('Are you sure you want to delete this project?')) {
    return;
  }

  try {
    await projectService.delete(project.value.id);
    emit('delete');
  } catch (error) {
    alert(`Error: ${error instanceof Error ? error.message : 'Failed to delete project'}`);
  }
};

const handleDisconnectGA4 = async () => {
  if (!confirm('Are you sure you want to disconnect from GA4?')) {
    return;
  }

  try {
    await ga4Service.disconnect(project.value.id);
    ga4Status.value = null;
  } catch (error) {
    ga4Error.value = error instanceof Error ? error.message : 'Failed to disconnect';
  }
};

const handlePullData = async () => {
  if (!ga4Status.value) return;

  try {
    await ga4Service.pullData(project.value.id, ga4Status.value.connector_id);
    alert('Data pull started successfully!');
  } catch (error) {
    ga4Error.value = error instanceof Error ? error.message : 'Failed to pull data';
  }
};

const fetchGA4Status = async () => {
  isLoadingGA4.value = true;
  ga4Error.value = '';

  try {
    const status = await ga4Service.getStatus(project.value.id);
    ga4Status.value = status;
  } catch (error) {
    console.error('Failed to fetch GA4 status:', error);
    ga4Error.value = error instanceof Error ? error.message : 'Failed to load GA4 status';
  } finally {
    isLoadingGA4.value = false;
  }
};

onMounted(() => {
  fetchGA4Status();
});
</script>

<style scoped>
.project-detail {
  background: white;
  border-radius: 8px;
  padding: 32px;
  min-height: 100vh;
}

.btn-back {
  background-color: #e0e0e0;
  color: #333;
  padding: 10px 16px;
  margin-bottom: 24px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-back:hover {
  background-color: #d0d0d0;
  transform: translateX(-2px);
}

.detail-container {
  max-width: 800px;
  margin: 0 auto;
}

.detail-header {
  border-bottom: 2px solid #f0f0f0;
  padding-bottom: 20px;
  margin-bottom: 30px;
}

.detail-header h1 {
  color: #333;
  font-size: 32px;
  margin-bottom: 12px;
}

.detail-id {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #666;
  font-size: 14px;
}

.detail-id code {
  background-color: #f5f5f5;
  padding: 4px 8px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  color: #e74c3c;
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

section h2 {
  font-size: 18px;
  color: #333;
  margin-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
  padding-bottom: 10px;
}

.description-section .description {
  color: #555;
  line-height: 1.6;
  font-size: 16px;
}

.no-description {
  color: #999;
  font-style: italic;
  font-size: 14px;
}

.connector-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.connector-loading {
  padding: 20px;
  text-align: center;
  color: #999;
  font-size: 14px;
}

.connector-link {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background-color: #f9f9f9;
  border: 2px solid #4a90e2;
  border-radius: 8px;
  text-decoration: none;
  color: #333;
  transition: all 0.3s ease;
  cursor: pointer;
}

.connector-link:hover {
  background-color: #f0f7ff;
  transform: translateX(4px);
  box-shadow: 0 4px 12px rgba(74, 144, 226, 0.2);
}

.connector-icon {
  font-size: 24px;
}

.connector-name {
  flex: 1;
  font-weight: 500;
  font-size: 16px;
}

.external-icon {
  color: #4a90e2;
  font-size: 14px;
}

.connector-connected {
  background-color: #f0f7ff;
  border: 2px solid #4a90e2;
  border-radius: 8px;
  padding: 16px;
}

.connection-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 16px;
}

.status-indicator {
  font-size: 10px;
}

.status-indicator.connected {
  color: #2e7d32;
  font-size: 16px;
}

.status-indicator.expired {
  color: #e65100;
  font-size: 16px;
}

.connection-header h3 {
  margin: 0;
  font-size: 16px;
  color: #333;
}

.connection-details {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid #d0e8ff;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
}

.detail-item .label {
  font-weight: 500;
  color: #666;
}

.detail-item .value {
  color: #333;
  font-weight: 500;
}

.connector-actions {
  display: flex;
  gap: 8px;
}

.actions-section {
  display: flex;
  gap: 12px;
  padding-top: 20px;
  border-top: 1px solid #f0f0f0;
}

.btn {
  padding: 12px 24px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  flex: 1;
}

.btn-sm {
  padding: 8px 16px;
  font-size: 13px;
  flex: auto;
}

.btn-primary {
  background-color: #4a90e2;
  color: white;
}

.btn-primary:hover {
  background-color: #357abd;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(74, 144, 226, 0.3);
}

.btn-danger {
  background-color: #e74c3c;
  color: white;
}

.btn-danger:hover {
  background-color: #c0392b;
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(231, 76, 60, 0.3);
}

.error-message {
  padding: 12px;
  background-color: #ffe0e0;
  color: #e74c3c;
  border-radius: 4px;
  margin-top: 16px;
  font-size: 14px;
}

.loading {
  text-align: center;
  padding: 60px 20px;
  color: #666;
  font-size: 16px;
}

@media (max-width: 640px) {
  .project-detail {
    padding: 16px;
  }

  .detail-header h1 {
    font-size: 24px;
  }

  .detail-content {
    gap: 20px;
  }

  .actions-section {
    flex-direction: column;
  }
}
</style>
