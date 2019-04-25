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
        stage('Check Formatting') {
            environment {
                CARGO_HOME = '/home/jenkins/.cargo'
                RUSTUP_HOME = '/home/jenkins/.rustup'
                RUSTFLAGS = "-D warnings"
            }
            agent {
                label 'linux'
            }
            steps {
                echo 'Checking formatting...'
                sh '$CARGO_HOME/bin/cargo fmt -- --check'
            }
        }
        stage('Run Tests') {
            parallel {
              
                stage("Test on Linux") {
                    agent {
			            docker {
			                image 'amethystrs/builder-linux:stable'
			                label 'docker'
			            } 
                    }
                    steps {
                        echo 'Beginning tests...'
                        sh 'cd rendy && cargo test --all --features "full vulkan"'
                        echo 'Tests done!'
                    }
                }
		                 stage("Test on macOS") {
                     environment {
                         CARGO_HOME = '/Users/jenkins/.cargo'
                         RUSTUP_HOME = '/Users/jenkins/.rustup'
                     }
                     agent {
                         label 'mac'
                     }
                     steps {
                         echo 'Beginning tests...'
                         sh 'cd rendy && cargo test --all --features "full metal"'
                         echo 'Tests done!'
                     }
                 }
            }
        }
    }
}
