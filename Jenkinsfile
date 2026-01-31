pipeline {
	agent {
		label 'k8s-agent'
	}
	environment {
		GIT_REPO = 'git@github.com:mendinuetaj/mega-uploader-auth.git'
		GIT_CREDENTIALS = 'github'
		DOCKER_IMAGE = 'mega-uploader-auth'
		DOCKER_TAG = "${env.GIT_TAG?:'latest'}"
		PLATFORMS = "linux/arm64"
		BUILDER = "multiarch-builder"
	}
	stages {
		stage('Checkout') {
			steps {
				echo "Checking out..."
				checkout([
					$class: 'GitSCM',
					branches: [[name: "*/${env.GIT_TAG?:'master'}"]],
					userRemoteConfigs: [[
						url: env.GIT_REPO,
						credentialsId: env.GIT_CREDENTIALS
					]]
				])
			}
		}
		stage('Build Docker Image') {
			steps {
				container('docker') {
					script {
						echo "Building Docker image for multiple architectures..."
						withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
							sh """
							set -e
							echo \$DOCKERHUB_PASS | docker login -u \$DOCKERHUB_USER --password-stdin

							# Create builder if it does not exist
							if ! docker buildx inspect ${BUILDER} >/dev/null 2>&1; then
								docker buildx create --name ${BUILDER} --driver docker-container --use
							else
								docker buildx use ${BUILDER}
							fi

							docker buildx inspect --bootstrap

							docker buildx build --platform ${PLATFORMS} \
								-t \$DOCKERHUB_USER/${DOCKER_IMAGE}:${DOCKER_TAG} \
								-t \$DOCKERHUB_USER/${DOCKER_IMAGE}:latest \
								--cache-from type=registry,ref=\$DOCKERHUB_USER/${DOCKER_IMAGE}:buildcache \
								--cache-to type=registry,ref=\$DOCKERHUB_USER/${DOCKER_IMAGE}:buildcache,mode=max \
								--push .
							"""
						}
					}
				}
			}
		}
		stage('Push Docker Image') {
			steps {
				echo "Skipping push as buildx already pushed the multi-arch image"
			}
		}
		stage('Deploy to K8s') {
			steps {
				container('kubectl') {
					withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
						withAWS(credentials: 'aws_creds_megaupload_serambiente_account', region: 'us-east-1') {
							script {
								echo "Deploying to Kubernetes..."
								sh "kubectl apply -f k8s/namespace.yaml"
								sh "kubectl apply -f k8s/configmap.yaml"
								sh "envsubst < k8s/aws-secret.yaml | kubectl apply -f -"
								sh "envsubst < k8s/deployment.yaml | kubectl apply -f -"
								sh "kubectl apply -f k8s/service.yaml"
								sh "kubectl apply -f k8s/ingress.yaml"
							}
						}
					}
				}
			}
		}
	}
	post {
		always {
			container('docker') {
				sh '''
				echo "🧹 Cleaning local Docker artifacts"
				docker buildx prune -f || true
				docker system prune -af || true
				'''
			}
		}
		success {
			echo "✅ Multi-arch build completed successfully"
		}
		failure {
			echo "❌ Build failed"
		}
	}
}