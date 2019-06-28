pipeline {
    agent none
    stages {
        stage("Pull new images") {
            agent {
                label 'docker'
            }
            steps {
                sh 'docker pull amethystrs/builder-linux:stable'
                sh 'docker pull amethystrs/builder-linux:nightly'
            }
        }
        stage('Cargo Fmt') {
            environment {
                RUSTFLAGS = "-D warnings"
            }
            agent {
                docker {
                    image 'amethystrs/builder-linux:stable'
                    label 'docker'
                }
            }
            steps {
                echo 'Checking formatting...'
                sh 'cargo fmt -- --check'
            }
        }
        stage('Cargo Check') {
            parallel {
                stage("stable") {
                    environment {
                        RUSTFLAGS = "-D warnings"
                    }
                    agent {
                        docker {
                            image 'amethystrs/builder-linux:stable'
                            label 'docker'
                        }
                    }
                    steps {
                        // Make sure static_assertion fails
                        echo 'Confirming static assertion...'
                        sh 'cargo check --all --all-targets || exit 0'

                        // Perform actual check
                        sh 'cargo check --all --all-targets --features "vulkan sdl_controller json saveload"'
                    }
                }
                stage("nightly") {
                    environment {
                        RUSTFLAGS = "-D warnings"
                    }
                    agent {
                        docker {
                            image 'amethystrs/builder-linux:nightly'
                            label 'docker'
                        }
                    }
                    steps {
                        // Make sure static_assertion fails
                        echo 'Confirming static assertion...'
                        sh 'cargo check --all --all-targets || exit 0'

                        // Perform actual check
                        echo 'Running Cargo check...'
                        sh 'cargo check --all --all-targets --features "nightly vulkan sdl_controller json saveload"'
                    }
                }
            }
        }
        // Separate stage for coverage to prevent race condition with the linux test stage (repo lock contention).
        stage('Coverage') {
            agent {
                docker {
                    image 'amethystrs/builder-linux:stable'
                    args '--privileged'
                    label 'docker'
                }
            }
            steps {
                withCredentials([string(credentialsId: 'codecov_token', variable: 'CODECOV_TOKEN')]) {
                    echo 'Calculating code coverage...'
                    sh './scripts/coverage.sh'
                    echo "Uploading coverage..."
                    sh "curl -s https://codecov.io/bash | bash -s ./target/coverage/merged -t $CODECOV_TOKEN"
                    echo "Uploaded code coverage!"
                }
            }
        }
        stage('Run Tests') {
            parallel {
                stage("Test on Windows") {
                    environment {
                        CARGO_HOME = 'C:\\Users\\root\\.cargo'
                        RUSTUP_HOME = 'C:\\Users\\root\\.rustup'
                    }
                    agent {
                        label 'windows'
                    }
                    steps {
                        echo 'Beginning tests...'
                        bat 'C:\\Users\\root\\.cargo\\bin\\cargo test --all --features "vulkan json saveload"'
                        echo 'Tests done!'
                    }
                }
                stage("Test on Linux") {
                    agent {
                        docker {
                            image 'amethystrs/builder-linux:stable'
                            label 'docker'
                        }
                    }
                    steps {
                        echo 'Beginning tests...'
                        sh 'cargo test --all --features "vulkan sdl_controller json saveload"'
                        echo 'Tests done!'
                    }
                }
                // macOS is commented out due to needing to upgrade the OS, but MacStadium did not do the original install with APFS so we cannot upgrade easily
                // stage("Test on macOS") {
                //     environment {
                //         CARGO_HOME = '/Users/jenkins/.cargo'
                //         RUSTUP_HOME = '/Users/jenkins/.rustup'
                //     }
                //     agent {
                //         label 'mac'
                //     }
                //     steps {
                //         echo 'Beginning tests...'
                //         sh '/Users/jenkins/.cargo/bin/cargo test --all --features "empty"'
                //         echo 'Tests done!'
                //     }
                // }
            }
        }
    }
    post {
        always {
            node('') {
                echo 'Cleaning up workspace'
                deleteDir()
                echo 'Workspace cleaned!'
            }
        }
    }
}

