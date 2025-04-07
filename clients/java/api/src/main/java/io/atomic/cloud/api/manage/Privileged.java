package io.atomic.cloud.api.manage;

import io.atomic.cloud.api.resource.CloudResource;
import io.atomic.cloud.api.resource.complex.CloudGroup;
import io.atomic.cloud.api.resource.complex.CloudNode;
import io.atomic.cloud.api.resource.complex.CloudServer;
import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudNode;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import io.atomic.cloud.api.transfer.Transfers;
import io.atomic.cloud.grpc.manage.Server;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

public interface Privileged {

    /**
     * This will return the transfer object in privileged mode
     * @return The transfer object
     */
    Transfers transfers();

    /**
     * This will request the controller to stop
     * After calling this method, this server will be killed or stopped
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> stopController();

    /**
     * Sets the status of a cloud resource
     * @param resource The resource to set group or node
     * @param active If the resource should be set to active or inactive
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> setResource(CloudResource resource, boolean active);

    /**
     * This will try to delete the provided resource from the controller
     * @param resource The resource to delete
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> deleteResource(CloudResource resource);

    /**
     * Creates a node on the controller
     * @param node Node to create
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> createNode(CloudNode node);

    /**
     * Creates a group on the controller
     * @param cloudGroup Group to create
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> createGroup(CloudGroup cloudGroup);

    /**
     * Tells the controller to start a server
     * NOTE: Make sure there is no server with the same name
     * @param priority The priority of the server
     * @param name The name of the server
     * @param node The node to start the server on
     * @param resources The resources that the server can use
     * @param spec The specs for the server
     * @return The id of the server
     */
    CompletableFuture<UUID> scheduleServer(
            int priority, String name, SimpleCloudNode node, Server.Resources resources, Server.Spec spec);

    /**
     * Writes data to the screen of a server
     * NOTE: The plugin driving the node must support this
     * @param server The server to write to
     * @param data The data to write
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<Void> writeToScreen(SimpleCloudServer server, byte[] data);

    /**
     * Resolves the node to a complex node
     * @param node The node to resolve
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<CloudNode> node(SimpleCloudNode node);

    /**
     * Resolves the group to a complex group
     * @param group The group to resolve
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<CloudGroup> group(SimpleCloudGroup group);

    /**
     * Resolves the server to a complex server
     * @param server The server to resolve
     * @return A future that completes when the controller handled the request
     */
    CompletableFuture<CloudServer> server(SimpleCloudServer server);
}
